//! Access rules per user, incident and compartment.

use std::collections::{HashMap, HashSet};

use ordo_core::{CompartmentId, IncidentId, UserId};

/// Decides who may read and write which incident compartment.
pub trait AccessPolicy: Send {
    /// Whether `user` may read `compartment` of `incident`. `roles` are the user's roles
    /// in this incident, from its roster (ADR-0046).
    fn can_read(
        &self,
        user: &UserId,
        incident: &IncidentId,
        compartment: &CompartmentId,
        roles: &[String],
    ) -> bool;
    /// Whether `user` may write an event of `module` to `compartment` of `incident`.
    fn can_write(
        &self,
        user: &UserId,
        incident: &IncidentId,
        compartment: &CompartmentId,
        module: &str,
        roles: &[String],
    ) -> bool;
    /// The app's views and functions `user` may use in `incident` with `roles`, e.g.
    /// `view_situation`, `edit_units`; the server sends them in the welcome.
    fn incident_permissions(
        &self,
        _user: &UserId,
        _incident: &IncidentId,
        _roles: &[String],
    ) -> Vec<String> {
        Vec::new()
    }
    /// Whether `user` may see and revoke the devices of everyone (not only their own).
    fn can_manage_devices(&self, _user: &UserId) -> bool {
        false
    }
    /// Whether `user` may plan incidents and grant roles in every incident.
    fn can_manage_incidents(&self, _user: &UserId) -> bool {
        false
    }
    /// Whether `user`, holding `roles` in `incident`, may grant and withdraw roles and
    /// change the incident's state.
    fn can_assign_roles(&self, _user: &UserId, _incident: &IncidentId, _roles: &[String]) -> bool {
        false
    }
}

/// A fixed table of compartments per user and per incident role (tests and prototype).
#[derive(Debug, Clone, Default)]
pub struct StaticPolicy {
    grants: HashMap<UserId, HashSet<CompartmentId>>,
    role_grants: HashMap<String, HashSet<CompartmentId>>,
    admins: HashSet<UserId>,
    planners: HashSet<UserId>,
    assigners: HashSet<String>,
}

impl StaticPolicy {
    /// Lets `user` read and write `compartments` in every incident.
    pub fn allow(mut self, user: &UserId, compartments: &[&str]) -> Self {
        let entry = self.grants.entry(user.clone()).or_default();
        for compartment in compartments {
            entry.insert(compartment.parse().expect("valid compartment ID"));
        }
        self
    }

    /// Lets everyone holding the incident role `role` read and write `compartments`.
    pub fn role(mut self, role: &str, compartments: &[&str]) -> Self {
        let entry = self.role_grants.entry(role.to_owned()).or_default();
        for compartment in compartments {
            entry.insert(compartment.parse().expect("valid compartment ID"));
        }
        self
    }

    /// Lets holders of the incident role `role` grant roles in their incident.
    pub fn assigner(mut self, role: &str) -> Self {
        self.assigners.insert(role.to_owned());
        self
    }

    /// Lets `user` manage the devices of everyone.
    pub fn admin(mut self, user: &UserId) -> Self {
        self.admins.insert(user.clone());
        self
    }

    /// Lets `user` plan incidents (and read their `incident` compartment).
    pub fn planner(mut self, user: &UserId) -> Self {
        self.planners.insert(user.clone());
        self
    }
}

impl AccessPolicy for StaticPolicy {
    fn can_manage_devices(&self, user: &UserId) -> bool {
        self.admins.contains(user)
    }

    fn can_manage_incidents(&self, user: &UserId) -> bool {
        self.planners.contains(user)
    }

    fn can_assign_roles(&self, _user: &UserId, _incident: &IncidentId, roles: &[String]) -> bool {
        roles.iter().any(|r| self.assigners.contains(r))
    }

    fn can_read(
        &self,
        user: &UserId,
        _: &IncidentId,
        compartment: &CompartmentId,
        roles: &[String],
    ) -> bool {
        self.grants
            .get(user)
            .is_some_and(|set| set.contains(compartment))
            || roles.iter().any(|r| {
                self.role_grants
                    .get(r)
                    .is_some_and(|set| set.contains(compartment))
            })
            || (self.planners.contains(user) && compartment.as_str() == "incident")
    }

    fn can_write(
        &self,
        user: &UserId,
        incident: &IncidentId,
        compartment: &CompartmentId,
        _module: &str,
        roles: &[String],
    ) -> bool {
        self.can_read(user, incident, compartment, roles)
    }
}
