-- The server's database at the first release (ADR-0004), as one baseline. IF NOT EXISTS
-- lets databases from before the first release adopt it; every later change is a new file.

-- Events exactly as ordered: signed, and sealed in protected compartments.
CREATE TABLE IF NOT EXISTS events (
    incident TEXT NOT NULL,
    compartment TEXT NOT NULL,
    seq INTEGER NOT NULL,
    event BLOB NOT NULL,
    PRIMARY KEY (incident, compartment, seq)
) STRICT;

-- Data keys of the streams, wrapped with the master key.
CREATE TABLE IF NOT EXISTS data_keys (
    incident TEXT NOT NULL,
    compartment TEXT NOT NULL,
    key_id TEXT NOT NULL,
    wrapped BLOB NOT NULL,
    PRIMARY KEY (incident, compartment)
) STRICT;

-- Server settings, e.g. its wrapped signing key.
CREATE TABLE IF NOT EXISTS settings (
    name TEXT PRIMARY KEY,
    value BLOB NOT NULL
) STRICT;

-- People who logged in with OpenID Connect (ADR-0038): the identity provider's subject maps
-- to a stable ORDO user ID; roles come from the provider's groups; `refresh` is the
-- provider's renewal token, wrapped with the master key (ADR-0040).
CREATE TABLE IF NOT EXISTS identities (
    issuer TEXT NOT NULL,
    subject TEXT NOT NULL,
    user_id TEXT NOT NULL UNIQUE,
    roles TEXT NOT NULL,
    refresh BLOB,
    PRIMARY KEY (issuer, subject)
) STRICT;

-- Paired devices and their public keys; the end of their login (ADR-0040) and when they
-- were revoked (SEC-4): revoked devices are refused for good.
CREATE TABLE IF NOT EXISTS devices (
    device_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    public_key BLOB NOT NULL,
    paired_ms INTEGER NOT NULL,
    valid_until INTEGER,
    revoked_ms INTEGER
) STRICT;

-- Attachments (ADR-0033): the stream each one belongs to; the encrypted bytes are in the
-- object store, named by their BLAKE3 hash.
CREATE TABLE IF NOT EXISTS blobs (
    hash TEXT NOT NULL,
    incident TEXT NOT NULL,
    compartment TEXT NOT NULL,
    size INTEGER NOT NULL,
    stored_ms INTEGER NOT NULL,
    PRIMARY KEY (hash, incident, compartment)
) STRICT;

-- Profile pictures per source: `ordo` (uploaded), `provider` (the identity provider's
-- `picture`), `gravatar` (ADR-0047).
CREATE TABLE IF NOT EXISTS avatars (
    user_id TEXT NOT NULL,
    source TEXT NOT NULL,
    image BLOB NOT NULL,
    updated_ms INTEGER NOT NULL,
    PRIMARY KEY (user_id, source)
) STRICT;
