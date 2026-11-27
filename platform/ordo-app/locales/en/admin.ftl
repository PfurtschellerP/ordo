tab-admin = Administration
tab-person = Person
ws-access = Access
ws-info = Info
devices = Devices
devices-intro-own = Your own paired devices. Block a lost or stolen device here: it is disconnected at once and deletes its data as soon as it connects again.
devices-intro-all = All paired devices of the organisation. Block a lost or stolen device here: it is disconnected at once and deletes its data as soon as it connects again.
devices-offline = The device list is available only with a connection to the server.
devices-loading = Loading the device list …
devices-empty = No paired devices.
this-device = this device
device-person = Person
device-paired = Paired
device-valid-until = Sign-in valid until
device-connection = Connection
device-connected = connected
device-not-connected = not connected
# Devices used in the incident, on *Vorbereitung – Personal* (ORG-12).
incident-devices = Devices in the incident
incident-devices-intro = The devices of everyone with a role in this incident. Block a lost or stolen device here: it is disconnected at once and deletes all ORDO data as soon as it connects again, also that of other incidents.
incident-devices-empty = No devices yet: nobody with a role in this incident has a paired device.
unknown = unknown
revoke = Block
revoke-confirm = Yes, block
device-name = Device …{ $short }
# "Firefox on Android · …4F2A9C": the device's name and the end of its ID (ORG-8).
named-device = { $name } · …{ $short }
revoke-label = Block: device …{ $short }

## Licences and source code (LIC-1)
notices-title = Licences and source code
notices-ordo-licence = ORDO is free software under the GNU Affero General Public License, version 3 or any later version (AGPL-3.0-or-later). It may be used, shared and changed; whoever offers a changed version over a network must offer its source code as well.
notices-licence-text = Licence text
notices-source = Source code of this version:
notices-parts = Other parts
notices-crates = Rust libraries
notices-crates-hint = The licence texts as they come with the libraries of server and app, each text with the libraries it applies to.
notices-crates-missing = The licence notices of the Rust libraries are not part of this build. Which libraries are used in which version is in the source code (Cargo.lock); official builds show their licence notices here.
info-version = Version
info-app = App
info-server = Server
info-build-local = local build
info-server-offline = The server cannot be reached right now.
info-loading = Asking …
# "Protocols: sync 1.0–1.2, HTTP 1.0"
info-protocols = Protocols: sync { $sync }, HTTP { $http }
notices-loading = Loading licence notices …
notices-used-map = Situation map
notices-used-map-font = Typeface of the map labels
notices-used-font = Typeface of the user interface
notices-used-icons = Icons of the user interface
notices-used-symbols = Tactical symbols (drawn when ORDO is built)
notices-used-storage = Data storage on the device
notices-used-map-data = Map data, where a base map is set up
notices-public-domain = public domain

## Organisation roles (ORG-9)
org-roles = Permissions
org-roles-title = Permissions in the organisation
org-roles-intro = Who administers the organisation or plans incidents. A permission applies on all the person's devices as soon as it is given, and ends when withdrawn or at the end of the chosen day. Roles in an incident are given by the incident command under “Preparation – Staff and roles”.
org-roles-loading = Loading roles …
org-roles-empty = No roles given yet.
org-roles-offline = Roles can be seen and changed only with a connection to the server.
org-role-grant-title = Give role
org-role-person = Person (user ID)
org-role-role = Role
org-role-until-label = Valid until (optional)
org-role-until-hint = Without a date, the role applies until it is withdrawn.
org-role-grant = Give
org-role-withdraw = Withdraw
org-role-invalid = Please give a user ID, e.g. anna@ordo.example.
org-role-past = The date is in the past.
org-role-fixed = from the server configuration
org-role-own-admin = your own administration cannot be withdrawn
org-role-lapsed = lapsed
# Until when a role counts, e.g. "until 30.11.2026".
org-role-until = until { $date }

## Local accounts (ORG-10)

## Upstream login providers (ORG-11, ADR-0063).
login-providers = Sign-in
providers-title = Sign-in through the organisation
providers-intro = Connect your organisation's sign-in service (OpenID Connect, e.g. Microsoft Entra ID, Keycloak, Authentik, Google). It then appears as a button of its own on the sign-in page. Before saving, you sign in through it once as a test.
providers-loading = Loading providers …
providers-not-managed = This server does not manage sign-in providers (no access to the sign-in service's interface). Providers are then in its configuration.
provider-local = Local accounts of this server
provider-change = Change
provider-delete = Remove
provider-delete-confirm = Yes, remove
provider-new = Add provider
provider-name = Name on the sign-in page
provider-name-hint = e.g. “Red Cross account”
provider-issuer = Issuer URL
provider-issuer-hint = e.g. https://login.microsoftonline.com/<tenant>/v2.0
provider-client-id = Client ID
provider-secret = Client secret
provider-secret-hint = Passed to the sign-in service and never shown in ORDO again.
provider-secret-keep = Leave empty to keep the secret so far.
provider-skip-email = Accept e-mail addresses without a verified flag too (e.g. with Microsoft Entra ID)
provider-redirect = Enter as the redirect URI at the provider:
provider-invalid = Please give a name, an issuer URL (https://…), a client ID and a client secret.
provider-test = Prepare test sign-in
provider-test-title = Test sign-in
provider-test-open = Open test sign-in
provider-test-hint = Come back here afterwards: the result appears here.
provider-save = Save provider
provider-saved = Provider saved.
provider-removed = Provider removed.
provider-kind = From the configuration ({ $kind })
provider-change-label = { provider-change }: { $name }
provider-delete-label = { provider-delete }: { $name }
provider-delete-question = Remove “{ $name }”? New sign-ins through it are then no longer possible.
provider-change-title = Change “{ $name }”
provider-test-waiting = Sign in through “{ $name }” in a new tab. The test sign-in is valid for 10 minutes; for that long the sign-in page also shows “Test: { $name }”.
provider-confirm = I confirm: sign-ins through “{ $name }” are accepted for this server.
provider-test-failed = The test sign-in failed: { $error }. Please check the details and test again.
accounts = Accounts
accounts-title = Sign-in accounts
accounts-intro = Accounts for signing in to this server, e.g. for helpers without an account of the organisation. Only the sign-in service stores passwords; ORDO does not know them.
accounts-loading = Loading accounts …
accounts-empty = No accounts yet.
accounts-offline = Accounts can be seen and changed only with a connection to the server.
accounts-not-managed = This server does not manage sign-in accounts (no access to the sign-in service's interface). Accounts then come from the configuration or the organisation's account system.
account-create-title = Create account
account-email = E-mail address (sign-in name)
account-name = Name
account-password = First password
account-new-password = New password
account-password-hint = At least 12 characters. Tell the person in person.
account-create = Create account
account-set-password = Set a new password
account-save-password = Save password
account-delete = Delete account
account-delete-confirm = Yes, delete account
account-done = Saved.
account-invalid = Please give an e-mail address and a name.
account-password-length = The password must have at least 12 characters (at most 72 bytes).
account-exists = There is already an account with this e-mail address.
account-not-found = This account no longer exists.
account-refused = The sign-in service refused the change. Accounts from its configuration can only be changed there.
account-unreachable = The sign-in service cannot be reached. Please try again later.
# The question before deleting an account.
account-delete-question = Delete the account of { $name } ({ $email })? The person can no longer sign in, and their devices are blocked.
# The accessible names of an account's buttons.
account-set-password-label = Set a new password: { $email }
account-delete-label = Delete account: { $email }

## Converted by hand: forms and values of ORDO's types
revoke-own-question = Block this device (…{ $short })? It is disconnected at once, and all data on it is deleted, also entries not sent yet.
revoke-other-question = Block device …{ $short }? It is disconnected at once and deletes all data on it as soon as it connects. Entries on the device not sent yet are lost.
# "0.0.0, build 1a2b3c4"
info-build-commit = { $version }, build { $commit }
info-build-local-of = { $version }, { info-build-local }
org-role-admin = Administration
org-role-planner = Incident planning (all incidents)
# The accessible name of the withdraw button. Variables: $role, $person
org-role-withdraw-of = Withdraw { $role }: { $person }
provider-test-passed-as = The test sign-in worked. ORDO would recognise it as { $user }.
# Variables: $user, $name, $email
provider-test-passed-name-email = The test sign-in worked. ORDO would recognise it as { $user } (name: { $name }, e-mail: { $email }).
provider-test-passed-name = The test sign-in worked. ORDO would recognise it as { $user } (name: { $name }).
provider-test-passed-email = The test sign-in worked. ORDO would recognise it as { $user } (e-mail: { $email }).
provider-not-found = This provider no longer exists, or it can only be changed in the configuration.
provider-not-tested = Please test successfully first; the test sign-in is valid for 10 minutes.
provider-refused = The sign-in service refused the change. Is its interface for providers enabled (DEX_API_CONNECTORS_CRUD)?

## The person's own pages (ADR-0100).
ws-person = Profile and devices
profile-view = Profile
