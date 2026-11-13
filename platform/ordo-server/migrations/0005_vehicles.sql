-- The organisation's vehicles added in ORDO (ORG-14, ADR-0068): master data offered when
-- units are added to an incident. Vehicles from ORDO_VEHICLES_FILE are not stored. The
-- strength is given in full (leaders, sub-leaders, crew) or not at all.
CREATE TABLE vehicles (
    unit TEXT PRIMARY KEY,
    unit_type TEXT NOT NULL,
    organisation TEXT NOT NULL,
    service TEXT,
    leaders INTEGER,
    sub_leaders INTEGER,
    crew INTEGER,
    changed_by TEXT NOT NULL,
    changed_at INTEGER NOT NULL
) STRICT;
