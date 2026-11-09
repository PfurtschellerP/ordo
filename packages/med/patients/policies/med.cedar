// Default roles of the Med package (ADR-0035). Organisations add or change roles in their
// own policy files (ORDO_POLICY_DIR) without changing code.

// Medical staff work with the shared incident data (UHS, counts) …
@id("med.medic-incident")
permit (
  principal,
  action == Action::"read",
  resource == Compartment::"incident"
)
when { context.roles.contains("medic") };

// … and write to it, except units: adding units and setting their status belongs to the
// incident command (edit_units).
@id("med.medic-incident-write")
permit (
  principal,
  action == Action::"write",
  resource == Compartment::"incident"
)
when { context.roles.contains("medic") && context.module != "units" };

// … and with clinical data: patients, triage, patient care records. Nobody else gets
// clinical data by default (need to know, SEC-9).
@id("med.medic-clinical")
permit (
  principal,
  action in [Action::"read", Action::"write"],
  resource == Compartment::"med.clinical"
)
when { context.roles.contains("medic") };
