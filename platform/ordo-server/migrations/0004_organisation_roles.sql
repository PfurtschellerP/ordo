-- Organisation roles granted in ORDO (ORG-9), each optionally until a time (ms since the
-- Unix epoch). Roles from the configuration are not stored.
CREATE TABLE organisation_roles (
    user_id TEXT NOT NULL,
    role TEXT NOT NULL,
    valid_until INTEGER,
    granted_by TEXT NOT NULL,
    granted_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, role)
) STRICT;
