// Default administration roles (ADR-0035). Organisations add or change roles in their own
// policy files (ORDO_POLICY_DIR) without changing code.

// Admins see and revoke the devices of everyone, e.g. a lost or stolen tablet (SEC-4).
// Everyone else manages only their own devices, which needs no policy.
@id("admin.manage-devices")
permit (
  principal,
  action == Action::"manage_devices",
  resource == Organisation::"organisation"
)
when { context.roles.contains("admin") };

// Admins grant and withdraw organisation roles, e.g. a planner for a season (ORG-9).
@id("admin.manage-roles")
permit (
  principal,
  action == Action::"manage_roles",
  resource == Organisation::"organisation"
)
when { context.roles.contains("admin") };

// Admins manage the local login accounts, e.g. for helpers joining on scene (ORG-10).
@id("admin.manage-accounts")
permit (
  principal,
  action == Action::"manage_accounts",
  resource == Organisation::"organisation"
)
when { context.roles.contains("admin") };

// Planners prepare incidents and their rosters (ADR-0046); administrators are planners
// too, so a new installation can start.
@id("admin.manage-incidents")
permit (
  principal,
  action == Action::"manage_incidents",
  resource == Organisation::"organisation"
)
when { context.roles.contains("planner") || context.roles.contains("admin") };

// Planners see and edit the shared data of every incident (name, roster, units), never
// clinical data, and use the incident command's views to prepare it, the map region too
// (ADR-0062).
@id("admin.planner-views")
permit (
  principal,
  action in [Action::"view_situation", Action::"edit_units", Action::"manage_map"],
  resource == Compartment::"incident"
)
when { context.roles.contains("planner") || context.roles.contains("admin") };

@id("admin.planner-incident")
permit (
  principal,
  action in [Action::"read", Action::"write"],
  resource == Compartment::"incident"
)
when { context.roles.contains("planner") || context.roles.contains("admin") };
