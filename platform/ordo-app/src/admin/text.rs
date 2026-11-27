//! User-visible texts of the administration package (ADR-0096). The messages are in
//! `locales/<language>/admin.ftl`; the build writes one function per message into this
//! module. What maps ORDO's values (errors, roles) to messages stays here. It re-exports
//! the shared modules' texts.

pub use crate::shared::text::*;

// [impl->dsn~ui-texts~2]
include!(concat!(env!("OUT_DIR"), "/text_admin.rs"));

pub fn revoke_question(short: &str, own: bool) -> String {
    if own {
        revoke_own_question(short)
    } else {
        revoke_other_question(short)
    }
}

/// "0.0.0, Build 1a2b3c4"
pub fn info_build(version: &str, commit: Option<&str>) -> String {
    match commit {
        Some(c) => info_build_commit(version, &c[..c.len().min(7)]),
        None => info_build_local_of(version),
    }
}

/// The label of an organisation role.
pub fn org_role(role: &str) -> &str {
    match role {
        "admin" => org_role_admin(),
        "planner" => org_role_planner(),
        other => other,
    }
}

/// The accessible name of the withdraw button.
pub fn org_role_withdraw_label(role: &str, person: &str) -> String {
    org_role_withdraw_of(org_role(role), person)
}

pub fn provider_test_passed(user: &str, name: Option<&str>, email: Option<&str>) -> String {
    match (name, email) {
        (Some(n), Some(e)) => provider_test_passed_name_email(user, n, e),
        (Some(n), None) => provider_test_passed_name(user, n),
        (None, Some(e)) => provider_test_passed_email(user, e),
        (None, None) => provider_test_passed_as(user),
    }
}

pub fn provider_error(error: ordo_sync::message::LoginProviderError) -> &'static str {
    use ordo_sync::message::LoginProviderError as E;
    match error {
        E::Invalid => provider_invalid(),
        E::NotFound => provider_not_found(),
        E::NotTested => provider_not_tested(),
        E::Refused => provider_refused(),
        E::Unreachable => account_unreachable(),
    }
}

/// Why a change to an account was not made.
pub fn account_error(error: ordo_sync::message::AccountError) -> &'static str {
    use ordo_sync::message::AccountError;
    match error {
        AccountError::Invalid => account_invalid(),
        AccountError::PasswordLength => account_password_length(),
        AccountError::Exists => account_exists(),
        AccountError::NotFound => account_not_found(),
        AccountError::Refused => account_refused(),
        AccountError::Unreachable => account_unreachable(),
    }
}

#[cfg(test)]
mod tests {
    // [utest->dsn~ui-texts~2]
    #[test]
    fn texts_by_hand_read_as_before() {
        use super::*;
        assert_eq!(
            info_build("0.0.0", Some("1a2b3c4d5e")),
            "0.0.0, Build 1a2b3c4"
        );
        assert_eq!(org_role("planner"), "Einsatzplanung (alle Einsätze)");
        assert_eq!(org_role("other"), "other");
        assert_eq!(
            provider_test_passed("ben@x", Some("Ben"), None),
            "Die Testanmeldung hat geklappt. ORDO würde sie als ben@x erkennen (Name: Ben)."
        );
    }
}
