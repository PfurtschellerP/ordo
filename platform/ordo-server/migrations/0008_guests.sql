-- Helpers without an account (ADR-0077): a guest ID for one incident, until when their
-- access lasts (their end of duty), and who made them. Their one-time codes are kept only as
-- SHA-256 hashes, valid once until `expires_ms`.
CREATE TABLE guests (
    user_id TEXT PRIMARY KEY,
    incident TEXT NOT NULL,
    until_ms INTEGER NOT NULL,
    created_by TEXT NOT NULL,
    created_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE guest_codes (
    hash BLOB PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES guests (user_id),
    expires_ms INTEGER NOT NULL,
    used_ms INTEGER
) STRICT;
