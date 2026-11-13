-- ORDO keeps no organisation-wide vehicle master data (ADR-0070): units are incident data,
-- planned per incident. Migration 0005 stays, because databases have applied it.
DROP TABLE IF EXISTS vehicles;
