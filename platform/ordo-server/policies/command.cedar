// Default roles of incident command (ADR-0035). Organisations add or change roles in their
// own policy files (ORDO_POLICY_DIR) without changing code.

// Commanders see and change the shared incident data: units, UHS, counts, situation.
@id("command.commander-incident")
permit (
  principal,
  action in [Action::"read", Action::"write"],
  resource == Compartment::"incident"
)
when { context.roles.contains("commander") };

// The incident command sees the situation of the whole incident (every UHS's load and
// counts, never clinical details), manages units, prepares the incident's map region
// (ADR-0062) and blocks the devices used in its incident (ORG-12).
@id("command.commander-views")
permit (
  principal,
  action in [
    Action::"view_situation",
    Action::"edit_units",
    Action::"manage_map",
    Action::"block_devices"
  ],
  resource == Compartment::"incident"
)
when { context.roles.contains("commander") };

// The incident command keeps the register of its helpers, with their personal data
// (HELP-1, ADR-0071); nobody else reads it.
@id("command.commander-people")
permit (
  principal,
  action in [Action::"read", Action::"write"],
  resource == Compartment::"people"
)
when { context.roles.contains("commander") };

// The incident command grants and withdraws roles in its incident (ADR-0046).
@id("command.commander-assigns-roles")
permit (
  principal,
  action == Action::"assign_role",
  resource == Compartment::"incident"
)
when { context.roles.contains("commander") };

// Helpers without an account (guests, ADR-0077) never lead an incident: a guest who holds
// the commander role loses every permission in it. A forbid wins over every permit, also
// over an organisation's own policies.
@id("command.guests-never-command")
forbid (
  principal,
  action,
  resource is Compartment
)
when { context.guest && context.roles.contains("commander") };

// Crew members without a medical role (Einsatzkraft, e.g. a driver; ADR-0101): on a unit's
// crew they read the shared incident data, set their unit's status and move their
// Aufträge on, in their own workspace (UNIT-9). Without a crew they read the shared data
// only. Never clinical data, never the helper register.
@id("command.crew-incident")
permit (
  principal,
  action == Action::"read",
  resource == Compartment::"incident"
)
when { context.roles.contains("crew") };

@id("command.crew-writes")
permit (
  principal,
  action == Action::"write",
  resource == Compartment::"incident"
)
when {
  context.roles.contains("crew") && context.on_crew &&
  ((context.module == "units" && context.event_type == "units.statusChanged") ||
   (context.module == "tasks" && context.event_type == "tasks.updated"))
};

@id("command.crew-work")
permit (
  principal,
  action == Action::"crew_work",
  resource == Compartment::"incident"
)
when { context.roles.contains("crew") && context.on_crew };
