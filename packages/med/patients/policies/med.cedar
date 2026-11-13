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

// … and write to it, except units, tasks, the command structure, the situation map and the
// hospital list: these belong to the incident command (edit_units, TASK-1, CMD-1, MAP-1,
// TRANS-2).
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
  context.module != "tasks" &&
  context.module != "command" &&
  context.module != "map" &&
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

// Medical orders are given only by physicians (ORDER-3, ADR-0053): under federal law a
// paramedic does not delegate medical measures. The whole crew acknowledges and completes
// them (orders.updated). An organisation whose rules differ changes this in its own policies.
@id("med.orders-given-by-physicians")
forbid (
  principal,
  action == Action::"write",
  resource == Compartment::"med.clinical"
)
when { context.event_type == "orders.given" }
unless { context.roles.contains("physician") };

// A spoken order recorded by the crew (orders.recorded) is confirmed only by those who give
// orders (ORDER-6); every crew role records one.
@id("med.orders-confirmed-by-physicians")
forbid (
  principal,
  action == Action::"write",
  resource == Compartment::"med.clinical"
)
when { context.event_type == "orders.confirmed" }
unless { context.roles.contains("physician") };

// The app offers the order form to them.
@id("med.give-orders")
permit (
  principal,
  action == Action::"give_orders",
  resource == Compartment::"incident"
)
when { context.roles.contains("physician") };

// Physicians work at every UHS, whatever its staff list (ADR-0056); everyone else writes the
// patients of a UHS with a staff list only when on it.
@id("med.all-uhs")
permit (
  principal,
  action == Action::"all_uhs",
  resource == Compartment::"incident"
)
when { context.roles.contains("physician") };
