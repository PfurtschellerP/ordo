-- When each device last connected (ORG-14): devices unused for 30 days are removed as if
-- revoked. Unknown for devices that have not connected since this migration; they count
-- from their pairing.
ALTER TABLE devices ADD COLUMN last_seen_ms INTEGER;
