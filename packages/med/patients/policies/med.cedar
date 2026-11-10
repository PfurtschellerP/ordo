// Default roles of the Med package (ADR-0035). Organisations add or change roles in their
// own policy files (ORDO_POLICY_DIR) without changing code.
//
// Roles: medic (Sanitätsdienst), physician (Arzt/Ärztin), paramedic (Notfallsanitäter/in).

// Medical staff work with the shared incident data (UHS, counts, transports) …
@id("med.medic-incident")
permit (
  principal,
  action == Action::"read",
  resource == Compartment::"incident"
)
when {
  context.roles.contains("medic") ||
  context.roles.contains("physician") ||
  context.roles.contains("paramedic")
};

// … and write to it, except units and the hospital list: adding units, setting their
// status and keeping the hospitals belong to the incident command (edit_units, TRANS-2).
@id("med.medic-incident-write")
permit (
  principal,
  action == Action::"write",
  resource == Compartment::"incident"
)
when {
  (context.roles.contains("medic") ||
   context.roles.contains("physician") ||
   context.roles.contains("paramedic")) &&
  context.module != "units" &&
  context.module != "med.hospitals"
};

// … and with clinical data: patients, triage, patient care records, orders. Nobody else
// gets clinical data by default (need to know, SEC-9).
@id("med.medic-clinical")
permit (
  principal,
  action in [Action::"read", Action::"write"],
  resource == Compartment::"med.clinical"
)
when {
  context.roles.contains("medic") ||
  context.roles.contains("physician") ||
  context.roles.contains("paramedic")
};

// Medical orders are given only by physicians and paramedics (ORDER-3); the whole crew
// acknowledges and completes them (orders.updated).
@id("med.orders-given-by-physicians-and-paramedics")
forbid (
  principal,
  action == Action::"write",
  resource == Compartment::"med.clinical"
)
when { context.event_type == "orders.given" }
unless { context.roles.contains("physician") || context.roles.contains("paramedic") };

// The app offers the order form to them.
@id("med.give-orders")
permit (
  principal,
  action == Action::"give_orders",
  resource == Compartment::"incident"
)
when { context.roles.contains("physician") || context.roles.contains("paramedic") };
