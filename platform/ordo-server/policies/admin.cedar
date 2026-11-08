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
