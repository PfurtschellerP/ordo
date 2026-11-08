//! Access rules per user, incident and compartment.

use std::collections::{HashMap, HashSet};

use ordo_core::{CompartmentId, IncidentId, UserId};

/// Decides who may read and write which incident compartment.
pub trait AccessPolicy: Send {
    /// Whether `user` may read `compartment` of `incident`.
    fn can_read(&self, user: &UserId, incident: &IncidentId, compartment: &CompartmentId) -> bool;
    /// Whether `user` may write to `compartment` of `incident`.
    fn can_write(&self, user: &UserId, incident: &IncidentId, compartment: &CompartmentId) -> bool;
    /// Whether `user` may see and revoke the devices of everyone (not only their own).
    fn can_manage_devices(&self, _user: &UserId) -> bool {
        false
    }
}

/// A fixed table of compartments per user, valid for all incidents (prototype).
#[derive(Debug, Clone, Default)]
pub struct StaticPolicy {
    grants: HashMap<UserId, HashSet<CompartmentId>>,
    admins: HashSet<UserId>,
}

impl StaticPolicy {
    /// Lets `user` read and write `compartments`.
    pub fn allow(mut self, user: &UserId, compartments: &[&str]) -> Self {
        let entry = self.grants.entry(user.clone()).or_default();
        for compartment in compartments {
            entry.insert(compartment.parse().expect("valid compartment ID"));
        }
        self
    }
}

impl StaticPolicy {
    /// Lets `user` manage the devices of everyone.
    pub fn admin(mut self, user: &UserId) -> Self {
        self.admins.insert(user.clone());
        self
    }
}

impl AccessPolicy for StaticPolicy {
    fn can_manage_devices(&self, user: &UserId) -> bool {
        self.admins.contains(user)
    }

    fn can_read(&self, user: &UserId, _: &IncidentId, compartment: &CompartmentId) -> bool {
        self.grants
            .get(user)
            .is_some_and(|set| set.contains(compartment))
    }

    fn can_write(&self, user: &UserId, incident: &IncidentId, compartment: &CompartmentId) -> bool {
        self.can_read(user, incident, compartment)
    }
}
