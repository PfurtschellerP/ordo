//! Kinds of event (MOD-2, ADR-0010): each module declares once, next to the payload, where
//! an event of a kind goes and which schema version its payload has. Writers record by kind,
//! so no shared table lists every module's events.

use crate::AppliedEvent;

// [impl->dsn~package-registry~1]
/// One kind of event a module writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventKind {
    /// The compartment it is written to, e.g. `med.clinical`.
    pub compartment: &'static str,
    /// The module, e.g. `med.patients`.
    pub module: &'static str,
    /// The event type, e.g. `patients.registered`.
    pub event_type: &'static str,
    /// The payload's schema version (1 until the payload first changes).
    pub schema: u32,
}

impl EventKind {
    /// A kind of event of `module` in `compartment`.
    pub const fn new(
        compartment: &'static str,
        module: &'static str,
        event_type: &'static str,
        schema: u32,
    ) -> Self {
        Self {
            compartment,
            module,
            event_type,
            schema,
        }
    }

    /// Whether `event` is of this kind (module and type; any schema version).
    pub fn matches(&self, event: &AppliedEvent) -> bool {
        event.body.module.as_str() == self.module
            && event.body.event_type.as_str() == self.event_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~package-registry~1]
    #[test]
    fn a_kind_names_where_an_event_goes_and_its_schema() {
        const REGISTERED: EventKind =
            EventKind::new("med.clinical", "med.patients", "patients.registered", 5);
        assert_eq!(
            (REGISTERED.compartment, REGISTERED.schema),
            ("med.clinical", 5)
        );
        assert_ne!(
            REGISTERED,
            EventKind::new("med.clinical", "med.patients", "patients.retriaged", 5)
        );
    }
}
