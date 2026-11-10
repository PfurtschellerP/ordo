//! Roles and permissions as Cedar policies (ADR-0035).
//!
//! People never hold permissions: a user has roles (prototype: from the seed file; later
//! from the identity provider and from positions in the command structure, ADR-0031), and
//! policies say what each role may do. Packages ship default policies; an organisation
//! adds roles and policies as `*.cedar` files in `ORDO_POLICY_DIR`, without changing code.
//! Every policy is validated against the schema when the server starts.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityId, EntityTypeName, EntityUid, PolicySet,
    Request, RestrictedExpression, Schema, ValidationMode, Validator,
};
use ordo_core::{CompartmentId, IncidentId, UserId};
use ordo_sync::AccessPolicy;

/// The schema every policy is checked against.
pub const SCHEMA: &str = include_str!("../policies/ordo.cedarschema");

/// Policies shipped with ORDO and its packages: (file name, text).
pub const DEFAULT_POLICIES: [(&str, &str); 3] = [
    ("admin.cedar", include_str!("../policies/admin.cedar")),
    ("command.cedar", include_str!("../policies/command.cedar")),
    (
        "med.cedar",
        include_str!("../../../packages/med/patients/policies/med.cedar"),
    ),
];

/// The app's views and functions decided per incident (Cedar actions on the incident's
/// shared compartment), see `ordo.cedarschema`.
pub const INCIDENT_PERMISSIONS: [&str; 3] = ["view_situation", "edit_units", "give_orders"];

/// A policy file could not be used.
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    /// The schema is broken (a bug in ORDO).
    #[error("authorisation schema: {0}")]
    Schema(String),
    /// A policy file does not parse or does not fit the schema.
    #[error("policy {file}: {message}")]
    Invalid {
        /// File name.
        file: String,
        /// What is wrong.
        message: String,
    },
}

/// The organisation roles of every user (administrators, planners; seed roles in
/// development). Roles within an incident come from its roster (ADR-0046).
#[derive(Clone, Default)]
pub struct Roles(Arc<RwLock<HashMap<UserId, Vec<String>>>>);

impl Roles {
    /// Roles from a list.
    pub fn new(roles: impl IntoIterator<Item = (UserId, Vec<String>)>) -> Self {
        Self(Arc::new(RwLock::new(roles.into_iter().collect())))
    }

    /// Adds roles of `user` (keeps the existing ones, no duplicates).
    pub fn add(&self, user: &UserId, roles: &[String]) {
        let mut map = self.0.write().expect("roles lock");
        let entry = map.entry(user.clone()).or_default();
        for role in roles {
            if !entry.contains(role) {
                entry.push(role.clone());
            }
        }
    }

    fn of(&self, user: &UserId) -> Vec<String> {
        self.0
            .read()
            .expect("roles lock")
            .get(user)
            .cloned()
            .unwrap_or_default()
    }
}

// [impl->dsn~role-policies~1]
/// Decides with Cedar policies, by the roles a user holds.
pub struct RolePolicy {
    schema: Schema,
    policies: PolicySet,
    authorizer: Authorizer,
    roles: Roles,
}

fn uid(entity_type: &str, id: &str) -> EntityUid {
    EntityUid::from_type_name_and_id(
        EntityTypeName::from_str(entity_type).expect("valid entity type"),
        EntityId::new(id),
    )
}

impl RolePolicy {
    /// Loads the default policies and `extra` ones (file name, text), validates all of
    /// them, and takes the roles of each user.
    pub fn new(roles: Roles, extra: &[(String, String)]) -> Result<Self, PolicyError> {
        let (schema, _warnings) =
            Schema::from_cedarschema_str(SCHEMA).map_err(|e| PolicyError::Schema(e.to_string()))?;
        let validator = Validator::new(schema.clone());
        let mut policies = PolicySet::new();
        let files = DEFAULT_POLICIES
            .iter()
            .map(|(f, t)| ((*f).to_owned(), (*t).to_owned()))
            .chain(extra.iter().cloned());
        for (file, text) in files {
            let invalid = |message: String| PolicyError::Invalid {
                file: file.clone(),
                message,
            };
            let set = PolicySet::from_str(&text).map_err(|e| invalid(e.to_string()))?;
            let result = validator.validate(&set, ValidationMode::Strict);
            if !result.validation_passed() {
                let messages: Vec<String> = result
                    .validation_errors()
                    .map(ToString::to_string)
                    .collect();
                return Err(invalid(messages.join("; ")));
            }
            // Policy IDs must be unique across files: prefix them with the file name.
            for policy in set.policies() {
                let id = cedar_policy::PolicyId::new(format!("{file}/{}", policy.id()));
                policies
                    .add(policy.new_id(id))
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        Ok(Self {
            schema,
            policies,
            authorizer: Authorizer::new(),
            roles,
        })
    }

    /// Whether `user`, holding `incident_roles` in `incident`, may do `action` on
    /// `compartment` of it. The user's organisation roles count as well.
    pub fn allows(
        &self,
        user: &UserId,
        action: &str,
        incident: &IncidentId,
        compartment: &CompartmentId,
        incident_roles: &[String],
    ) -> bool {
        self.allows_write(
            user,
            action,
            incident,
            compartment,
            ("", ""),
            incident_roles,
        )
    }

    // [impl->dsn~order-permission~1]
    /// Like [`Self::allows`], for writing an event of `(module, event type)`, e.g.
    /// `("med.orders", "orders.given")`.
    pub fn allows_write(
        &self,
        user: &UserId,
        action: &str,
        incident: &IncidentId,
        compartment: &CompartmentId,
        (module, event_type): (&str, &str),
        incident_roles: &[String],
    ) -> bool {
        let mut roles = self.roles.of(user);
        roles.extend(incident_roles.iter().cloned());
        let context = Context::from_pairs([
            (
                "roles".to_owned(),
                RestrictedExpression::new_set(
                    roles.into_iter().map(RestrictedExpression::new_string),
                ),
            ),
            (
                "incident".to_owned(),
                RestrictedExpression::new_string(incident.to_string()),
            ),
            (
                "module".to_owned(),
                RestrictedExpression::new_string(module.to_owned()),
            ),
            (
                "event_type".to_owned(),
                RestrictedExpression::new_string(event_type.to_owned()),
            ),
        ]);
        let Ok(context) = context else { return false };
        let request = Request::new(
            uid("User", user.as_str()),
            uid("Action", action),
            uid("Compartment", compartment.as_str()),
            context,
            Some(&self.schema),
        );
        let Ok(request) = request else { return false };
        self.authorizer
            .is_authorized(&request, &self.policies, &Entities::empty())
            .decision()
            == Decision::Allow
    }
}

impl RolePolicy {
    /// Whether `user` may do the organisation-wide `action`, e.g. `manage_devices`.
    pub fn allows_organisation(&self, user: &UserId, action: &str) -> bool {
        let roles = self.roles.of(user);
        let context = Context::from_pairs([(
            "roles".to_owned(),
            RestrictedExpression::new_set(roles.into_iter().map(RestrictedExpression::new_string)),
        )]);
        let Ok(context) = context else { return false };
        let Ok(request) = Request::new(
            uid("User", user.as_str()),
            uid("Action", action),
            uid("Organisation", "organisation"),
            context,
            Some(&self.schema),
        ) else {
            return false;
        };
        self.authorizer
            .is_authorized(&request, &self.policies, &Entities::empty())
            .decision()
            == Decision::Allow
    }
}

impl AccessPolicy for RolePolicy {
    fn can_manage_devices(&self, user: &UserId) -> bool {
        self.allows_organisation(user, "manage_devices")
    }

    fn can_manage_incidents(&self, user: &UserId) -> bool {
        self.allows_organisation(user, "manage_incidents")
    }

    // [impl->dsn~incident-roles~1]
    fn can_assign_roles(&self, user: &UserId, incident: &IncidentId, roles: &[String]) -> bool {
        let shared: CompartmentId = "incident".parse().expect("valid compartment");
        self.allows(user, "assign_role", incident, &shared, roles)
    }

    fn can_read(
        &self,
        user: &UserId,
        incident: &IncidentId,
        compartment: &CompartmentId,
        roles: &[String],
    ) -> bool {
        self.allows(user, "read", incident, compartment, roles)
    }

    fn can_write(
        &self,
        user: &UserId,
        incident: &IncidentId,
        compartment: &CompartmentId,
        module: &str,
        event_type: &str,
        roles: &[String],
    ) -> bool {
        self.allows_write(
            user,
            "write",
            incident,
            compartment,
            (module, event_type),
            roles,
        )
    }

    // [impl->dsn~views-by-role~1]
    fn incident_permissions(
        &self,
        user: &UserId,
        incident: &IncidentId,
        roles: &[String],
    ) -> Vec<String> {
        let shared: CompartmentId = "incident".parse().expect("valid compartment");
        INCIDENT_PERMISSIONS
            .iter()
            .filter(|action| self.allows(user, action, incident, &shared, roles))
            .map(|action| (*action).to_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::{ServerName, Ulid};

    fn user(name: &str) -> UserId {
        format!("{name}@ordo.example.org").parse().unwrap()
    }

    fn incident() -> IncidentId {
        let server: ServerName = "ordo.example.org".parse().unwrap();
        IncidentId::new(Ulid::from_parts(1, 1).unwrap(), &server)
    }

    fn compartment(name: &str) -> CompartmentId {
        name.parse().unwrap()
    }

    fn policy(extra: &[(String, String)]) -> Result<RolePolicy, PolicyError> {
        let roles = Roles::new([
            (user("anna"), vec!["medic".to_owned()]),
            (user("ben"), vec!["commander".to_owned()]),
            (user("clara"), vec!["uhs_lead".to_owned()]),
        ]);
        RolePolicy::new(roles, extra)
    }

    // [utest->dsn~role-policies~1]
    #[test]
    fn default_roles_follow_need_to_know() {
        let p = policy(&[]).expect("default policies are valid");
        let (incident, shared, clinical) = (
            incident(),
            compartment("incident"),
            compartment("med.clinical"),
        );
        assert!(
            p.can_read(&user("anna"), &incident, &clinical, &[]),
            "medics read clinical data"
        );
        assert!(p.can_write(&user("anna"), &incident, &shared, "", "", &[]));
        assert!(
            p.can_read(&user("ben"), &incident, &shared, &[]),
            "commanders read incident data"
        );
        assert!(
            !p.can_read(&user("ben"), &incident, &clinical, &[]),
            "but no clinical data"
        );
        assert!(
            !p.can_read(&user("clara"), &incident, &shared, &[]),
            "unknown role: nothing"
        );
        assert!(
            !p.can_read(&user("nobody"), &incident, &shared, &[]),
            "no roles: nothing"
        );
        assert!(
            !p.can_manage_devices(&user("anna")),
            "medics manage only their own devices"
        );
    }

    // [utest->dsn~device-revocation~1]
    #[test]
    fn admins_manage_all_devices() {
        let roles = Roles::new([(user("dora"), vec!["admin".to_owned()])]);
        let p = RolePolicy::new(roles, &[]).expect("valid");
        assert!(p.can_manage_devices(&user("dora")));
        assert!(
            !p.can_read(
                &user("dora"),
                &incident(),
                &compartment("med.clinical"),
                &[]
            ),
            "an admin is not a medic"
        );
    }

    // [utest->dsn~incident-roles~1]
    #[test]
    fn roles_in_the_incident_decide_and_planners_plan() {
        let roles = Roles::new([(user("dora"), vec!["admin".to_owned()])]);
        let p = RolePolicy::new(roles, &[]).expect("valid");
        let (incident, shared, clinical) = (
            incident(),
            compartment("incident"),
            compartment("med.clinical"),
        );
        let medic = ["medic".to_owned()];
        let commander = ["commander".to_owned()];
        let erik = user("erik");
        assert!(
            !p.can_read(&erik, &incident, &clinical, &[]),
            "no role in the incident: nothing"
        );
        assert!(
            p.can_read(&erik, &incident, &clinical, &medic),
            "a medic in this incident reads its clinical data"
        );
        assert!(!p.can_assign_roles(&erik, &incident, &medic));
        assert!(p.can_assign_roles(&erik, &incident, &commander));
        assert!(!p.can_manage_incidents(&erik));
        assert!(p.can_manage_incidents(&user("dora")), "administrators plan");
        assert!(p.can_read(&user("dora"), &incident, &shared, &[]));
        assert!(
            !p.can_read(&user("dora"), &incident, &clinical, &[]),
            "planning gives no clinical data"
        );
    }

    // [utest->dsn~views-by-role~1]
    #[test]
    fn views_and_unit_changes_follow_the_role() {
        let p = policy(&[]).expect("valid");
        let (incident, shared) = (incident(), compartment("incident"));
        let erik = user("erik");
        let medic = ["medic".to_owned()];
        let commander = ["commander".to_owned()];
        assert!(
            p.incident_permissions(&erik, &incident, &medic).is_empty(),
            "UHS crews: no situation view"
        );
        assert_eq!(
            p.incident_permissions(&erik, &incident, &commander),
            ["view_situation", "edit_units"]
        );
        assert_eq!(
            p.incident_permissions(&erik, &incident, &["physician".to_owned()]),
            ["give_orders"],
            "physicians give orders"
        );
        assert!(
            p.can_write(&erik, &incident, &shared, "med.uhs", "uhs.created", &medic),
            "medics manage UHS"
        );
        assert!(
            !p.can_write(&erik, &incident, &shared, "units", "units.added", &medic),
            "but not units"
        );
        assert!(p.can_write(
            &erik,
            &incident,
            &shared,
            "units",
            "units.added",
            &commander
        ));
        // [utest->dsn~hospital-list~1]
        assert!(
            !p.can_write(
                &erik,
                &incident,
                &shared,
                "med.hospitals",
                "hospitals.capacitySet",
                &medic
            ),
            "medics read the hospitals, the command keeps them"
        );
        assert!(p.can_write(
            &erik,
            &incident,
            &shared,
            "med.hospitals",
            "hospitals.set",
            &commander
        ));
        // [utest->dsn~tasks~1]
        assert!(
            !p.can_write(&erik, &incident, &shared, "tasks", "tasks.given", &medic),
            "tasks are the command's"
        );
        assert!(p.can_write(
            &erik,
            &incident,
            &shared,
            "tasks",
            "tasks.given",
            &commander
        ));
        // [utest->dsn~command-structure~1]
        assert!(
            !p.can_write(
                &erik,
                &incident,
                &shared,
                "command",
                "command.positionCreated",
                &medic
            ),
            "the command structure is the command's"
        );
        assert!(p.can_write(
            &erik,
            &incident,
            &shared,
            "command",
            "command.leaderSet",
            &commander
        ));
    }

    // [utest->dsn~role-policies~1]
    #[test]
    fn organisations_add_roles_without_code() {
        let custom = r#"
            @id("uhs-lead")
            permit (principal, action == Action::"read", resource)
            when {
              context.roles.contains("uhs_lead") &&
              [Compartment::"incident", Compartment::"med.clinical"].contains(resource)
            };
        "#;
        let p = policy(&[("org.cedar".to_owned(), custom.to_owned())]).expect("valid");
        assert!(p.can_read(
            &user("clara"),
            &incident(),
            &compartment("med.clinical"),
            &[]
        ));
        assert!(!p.can_write(
            &user("clara"),
            &incident(),
            &compartment("med.clinical"),
            "",
            "",
            &[]
        ));
    }

    // [utest->dsn~role-policies~1]
    #[test]
    fn mistakes_in_policies_stop_the_server() {
        let typo =
            r#"permit (principal, action, resource) when { context.role.contains("medic") };"#;
        let error = policy(&[("org.cedar".to_owned(), typo.to_owned())]).err();
        assert!(
            matches!(&error, Some(PolicyError::Invalid { file, .. }) if file == "org.cedar"),
            "{error:?}"
        );
        assert!(policy(&[("broken.cedar".to_owned(), "permit (".to_owned())]).is_err());
    }

    // [utest->dsn~order-permission~1]
    #[test]
    fn only_physicians_and_paramedics_give_orders_the_crew_carries_them_out() {
        let p = policy(&[]).expect("valid");
        let (incident, clinical) = (incident(), compartment("med.clinical"));
        let erik = user("erik");
        let write = |roles: &[&str], event_type: &str| {
            let roles: Vec<String> = roles.iter().map(|r| (*r).to_owned()).collect();
            p.can_write(
                &erik,
                &incident,
                &clinical,
                "med.orders",
                event_type,
                &roles,
            )
        };
        assert!(write(&["physician"], "orders.given"));
        assert!(write(&["paramedic"], "orders.given"));
        assert!(
            !write(&["medic"], "orders.given"),
            "a medic gives no orders"
        );
        assert!(
            write(&["medic"], "orders.updated"),
            "but acknowledges and completes them"
        );
        assert!(write(&["medic", "physician"], "orders.given"));
        assert!(!write(&[], "orders.updated"));
        for role in ["physician", "paramedic"] {
            let roles = [role.to_owned()];
            assert!(
                p.can_read(&erik, &incident, &clinical, &roles),
                "{role} reads clinical data"
            );
            assert!(
                p.can_write(
                    &erik,
                    &incident,
                    &clinical,
                    "med.epcr",
                    "epcr.entryRecorded",
                    &roles
                ),
                "{role} documents"
            );
        }
    }
}
