-- Streams deleted at the end of their retention period (HELP-4, ADR-0071): their events and
-- data key are gone; this note keeps them from being written again and tells devices to
-- delete their copy.
CREATE TABLE deleted_streams (
    incident TEXT NOT NULL,
    compartment TEXT NOT NULL,
    deleted_ms INTEGER NOT NULL,
    PRIMARY KEY (incident, compartment)
) STRICT;
