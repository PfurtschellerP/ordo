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
// counts, never clinical details), manages units and prepares the incident's map region
// (ADR-0062).
@id("command.commander-views")
permit (
  principal,
  action in [Action::"view_situation", Action::"edit_units", Action::"manage_map"],
  resource == Compartment::"incident"
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
