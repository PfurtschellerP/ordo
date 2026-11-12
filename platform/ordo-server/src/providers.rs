//! Upstream login providers (ORG-11, ADR-0063): OpenID Connect connectors of Dex, written
//! through Dex's connector API after a test login through a temporary connector.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ordo_core::UserId;
use ordo_sync::message::{LoginProvider, LoginTest, LoginTestState};

use crate::dex_api::Connector;

/// IDs of temporary connectors start with this.
pub(crate) const TEST_PREFIX: &str = "ordo-test-";
/// How long a test login may take.
pub(crate) const TEST_TTL: Duration = Duration::from_secs(10 * 60);
/// Dex's connector of its local accounts.
const LOCAL: &str = "local";

/// A test login through a temporary connector.
pub(crate) struct Test {
    /// The temporary connector's ID.
    pub connector: String,
    /// The provider being changed; none for a new one.
    pub target: Option<String>,
    /// The provider's name.
    pub name: String,
    /// Dex's `oidc` configuration, with the secret.
    pub config: Vec<u8>,
    /// Who started it.
    pub by: UserId,
    started: Instant,
    /// How it went.
    pub state: LoginTestState,
}

/// The running test logins, by token.
#[derive(Default)]
pub(crate) struct Tests(Mutex<HashMap<String, Test>>);

impl Tests {
    /// Starts a test; returns the connectors of the user's earlier test, to delete.
    pub(crate) fn insert(&self, token: String, test: NewTest) -> Vec<String> {
        let mut tests = self.0.lock().expect("login tests");
        let earlier: Vec<String> = tests
            .iter()
            .filter(|(_, t)| t.by == test.by)
            .map(|(k, _)| k.clone())
            .collect();
        let gone = earlier
            .iter()
            .filter_map(|k| tests.remove(k))
            .map(|t| t.connector)
            .collect();
        tests.insert(
            token,
            Test {
                connector: test.connector,
                target: test.target,
                name: test.name,
                config: test.config,
                by: test.by,
                started: Instant::now(),
                state: LoginTestState::Waiting,
            },
        );
        gone
    }

    /// The temporary connector of a test that still waits for its login.
    pub(crate) fn waiting(&self, token: &str) -> Option<String> {
        let tests = self.0.lock().expect("login tests");
        tests
            .get(token)
            .filter(|t| t.started.elapsed() < TEST_TTL && t.state == LoginTestState::Waiting)
            .map(|t| t.connector.clone())
    }

    /// Records how the test login went.
    pub(crate) fn finish(&self, token: &str, state: LoginTestState) {
        if let Some(t) = self.0.lock().expect("login tests").get_mut(token) {
            t.state = state;
        }
    }

    /// How a test went.
    pub(crate) fn state(&self, token: &str) -> Option<LoginTestState> {
        let tests = self.0.lock().expect("login tests");
        tests.get(token).map(|t| t.state.clone())
    }

    /// The user's test, as the app shows it.
    pub(crate) fn of(&self, user: &UserId) -> Option<LoginTest> {
        let tests = self.0.lock().expect("login tests");
        tests
            .iter()
            .find(|(_, t)| &t.by == user && t.started.elapsed() < TEST_TTL)
            .map(|(token, t)| LoginTest {
                token: token.clone(),
                url: format!("/auth/test/{token}"),
                name: t.name.clone(),
                state: t.state.clone(),
            })
    }

    /// Takes the user's passed test, to save it.
    pub(crate) fn take_passed(&self, token: &str, user: &UserId) -> Option<Test> {
        let mut tests = self.0.lock().expect("login tests");
        let passed = tests.get(token).is_some_and(|t| {
            &t.by == user
                && t.started.elapsed() < TEST_TTL
                && matches!(t.state, LoginTestState::Passed { .. })
        });
        passed.then(|| tests.remove(token)).flatten()
    }

    /// Drops expired tests; returns the temporary connectors still in use, and whether
    /// any test expired.
    pub(crate) fn expire(&self) -> (Vec<String>, bool) {
        let mut tests = self.0.lock().expect("login tests");
        let before = tests.len();
        tests.retain(|_, t| t.started.elapsed() < TEST_TTL);
        let live = tests.values().map(|t| t.connector.clone()).collect();
        (live, tests.len() < before)
    }
}

/// What a new test needs.
pub(crate) struct NewTest {
    pub connector: String,
    pub target: Option<String>,
    pub name: String,
    pub config: Vec<u8>,
    pub by: UserId,
}

// [impl->dsn~login-providers~1]
/// Dex's `oidc` configuration of a provider. The scopes ask for the identity only: roles
/// come from ORDO (ADR-0046).
pub(crate) fn oidc_config(
    issuer: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    skip_email_verified: bool,
) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "issuer": issuer,
        "clientID": client_id,
        "clientSecret": client_secret,
        "redirectURI": redirect_uri,
        "scopes": ["openid", "profile", "email"],
        "insecureSkipEmailVerified": skip_email_verified,
    }))
    .expect("JSON")
}

/// The client secret in a connector's configuration.
pub(crate) fn secret_of(connector: &Connector) -> Option<String> {
    let config: serde_json::Value = serde_json::from_slice(&connector.config).ok()?;
    config["clientSecret"].as_str().map(str::to_owned)
}

// [impl->dsn~login-providers~1]
/// A connector as the app sees it: never with its secret; temporary ones are left out.
pub(crate) fn provider_of(connector: &Connector) -> Option<LoginProvider> {
    if connector.id.starts_with(TEST_PREFIX) {
        return None;
    }
    let config: serde_json::Value = serde_json::from_slice(&connector.config).unwrap_or_default();
    let oidc = connector.r#type == "oidc";
    let text = |key: &str| {
        oidc.then(|| config[key].as_str().map(str::to_owned))
            .flatten()
    };
    Some(LoginProvider {
        id: connector.id.clone(),
        name: connector.name.clone(),
        kind: connector.r#type.clone(),
        issuer: text("issuer"),
        client_id: text("clientID"),
        skip_email_verified: oidc && config["insecureSkipEmailVerified"].as_bool() == Some(true),
        editable: oidc && connector.id != LOCAL,
    })
}

/// A connector ID from a provider's name, unlike the IDs in `taken`: lower-case letters,
/// digits and `-`, at most 40 characters.
pub(crate) fn id_for(name: &str, taken: &[String]) -> String {
    let mut slug = String::new();
    for c in name.trim().to_lowercase().chars() {
        let c = match c {
            'ä' => "ae".to_owned(),
            'ö' => "oe".to_owned(),
            'ü' => "ue".to_owned(),
            'ß' => "ss".to_owned(),
            c if c.is_ascii_alphanumeric() => c.to_string(),
            _ => "-".to_owned(),
        };
        if !(c == "-" && (slug.is_empty() || slug.ends_with('-'))) {
            slug.push_str(&c);
        }
    }
    let mut slug: String = slug.trim_end_matches('-').chars().take(40).collect();
    if slug.is_empty() || slug == LOCAL || slug.starts_with(TEST_PREFIX) {
        slug = format!("provider-{slug}").trim_end_matches('-').to_owned();
    }
    let mut id = slug.clone();
    let mut n = 2;
    while taken.contains(&id) {
        id = format!("{slug}-{n}");
        n += 1;
    }
    id
}

/// Whether ORDO may change or delete this connector.
pub(crate) fn changeable(id: &str) -> bool {
    id != LOCAL && !id.starts_with(TEST_PREFIX)
}

/// The issuer URL as typed, if it is an http(s) URL without query or fragment.
pub(crate) fn issuer(text: &str) -> Option<String> {
    let text = text.trim().trim_end_matches('/');
    let url = reqwest::Url::parse(text).ok()?;
    (matches!(url.scheme(), "https" | "http")
        && url.host().is_some()
        && url.query().is_none()
        && url.fragment().is_none())
    .then(|| text.to_owned())
}

/// A small page for the browser tab of a test login (no scripts, no styles).
pub(crate) fn test_page(state: &LoginTestState) -> String {
    let escape = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    };
    let (title, text) = match state {
        LoginTestState::Passed { user, .. } => (
            "Testanmeldung erfolgreich",
            format!(
                "ORDO würde diese Anmeldung als <strong>{}</strong> erkennen. Dieses Fenster kann geschlossen werden; gespeichert wird in ORDO unter „Anmeldung“.",
                escape(user)
            ),
        ),
        LoginTestState::Failed(error) => (
            "Testanmeldung fehlgeschlagen",
            format!(
                "{} Dieses Fenster kann geschlossen werden; ORDO zeigt den Fehler unter „Anmeldung“.",
                escape(error)
            ),
        ),
        LoginTestState::Waiting => (
            "Testanmeldung abgelaufen",
            "Bitte die Testanmeldung in ORDO neu starten.".to_owned(),
        ),
    };
    format!(
        "<!doctype html><html lang=\"de\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>ORDO – {title}</title></head><body><main><h1>{title}</h1><p>{text}</p></main></body></html>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(name: &str) -> UserId {
        format!("{name}@ordo.example.org").parse().unwrap()
    }

    // [utest->dsn~login-providers~1]
    #[test]
    fn the_app_sees_providers_without_secrets_and_ids_follow_the_name() {
        let config = oidc_config(
            "https://login.example.org",
            "ordo",
            "s3cret",
            "https://ordo.example.org/dex/callback",
            true,
        );
        let connector = Connector {
            id: "drk-konto".into(),
            r#type: "oidc".into(),
            name: "DRK-Konto".into(),
            config,
            grant_types: Vec::new(),
        };
        let p = provider_of(&connector).unwrap();
        assert_eq!(
            (
                p.issuer.as_deref(),
                p.client_id.as_deref(),
                p.skip_email_verified,
                p.editable
            ),
            (Some("https://login.example.org"), Some("ordo"), true, true)
        );
        assert!(!format!("{p:?}").contains("s3cret"), "never the secret");
        assert_eq!(secret_of(&connector).as_deref(), Some("s3cret"));
        let local = Connector {
            id: "local".into(),
            r#type: "local".into(),
            name: "Email".into(),
            ..Default::default()
        };
        assert!(
            !provider_of(&local).unwrap().editable,
            "local accounts stay"
        );
        let test = Connector {
            id: format!("{TEST_PREFIX}ab12"),
            ..Default::default()
        };
        assert!(
            provider_of(&test).is_none(),
            "temporary ones are not listed"
        );

        assert_eq!(
            id_for("DRK-Konto (Kreisverband)", &[]),
            "drk-konto-kreisverband"
        );
        assert_eq!(id_for("Größe & Mähren", &[]), "groesse-maehren");
        assert_eq!(id_for("Local", &[]), "provider-local");
        assert_eq!(id_for("  ", &[]), "provider");
        assert_eq!(id_for("ASB", &["asb".into(), "asb-2".into()]), "asb-3");
        assert!(!changeable("local") && !changeable(&format!("{TEST_PREFIX}x")));

        assert_eq!(
            issuer("https://login.example.org/tenant/v2.0/").as_deref(),
            Some("https://login.example.org/tenant/v2.0")
        );
        assert!(issuer("ftp://x").is_none() && issuer("https://x/?a=1").is_none());
        assert!(issuer("keine URL").is_none());
    }

    // [utest->dsn~login-providers~1]
    #[test]
    fn only_a_passed_test_of_the_same_person_is_saved() {
        let tests = Tests::default();
        let new = |connector: &str, by: &str| NewTest {
            connector: connector.into(),
            target: None,
            name: "DRK".into(),
            config: Vec::new(),
            by: user(by),
        };
        assert!(
            tests
                .insert("t1".into(), new("ordo-test-1", "dora"))
                .is_empty()
        );
        assert_eq!(tests.waiting("t1").as_deref(), Some("ordo-test-1"));
        assert!(
            tests.take_passed("t1", &user("dora")).is_none(),
            "not passed yet"
        );
        tests.finish(
            "t1",
            LoginTestState::Passed {
                user: "olga@ordo.example.org".into(),
                name: None,
                email: None,
            },
        );
        assert!(tests.waiting("t1").is_none(), "one login per test");
        assert!(
            tests.take_passed("t1", &user("eva")).is_none(),
            "not someone else's"
        );
        assert_eq!(tests.of(&user("dora")).unwrap().url, "/auth/test/t1");
        assert!(tests.take_passed("t1", &user("dora")).is_some());
        assert!(tests.of(&user("dora")).is_none(), "taken");

        // A new test of the same person replaces the earlier one.
        tests.insert("t2".into(), new("ordo-test-2", "dora"));
        assert_eq!(
            tests.insert("t3".into(), new("ordo-test-3", "dora")),
            ["ordo-test-2"]
        );
        assert_eq!(tests.expire(), (vec!["ordo-test-3".to_owned()], false));
    }

    #[test]
    fn the_test_page_escapes_what_the_provider_sent() {
        let page = test_page(&LoginTestState::Passed {
            user: "<script>@x".into(),
            name: None,
            email: None,
        });
        assert!(page.contains("&lt;script&gt;") && !page.contains("<script>"));
    }
}
