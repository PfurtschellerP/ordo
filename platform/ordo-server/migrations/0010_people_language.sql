-- The language a person chose for ORDO's interface (I18N-1, ADR-0100): it follows them to
-- every device. None: not chosen, the device follows the browser.
ALTER TABLE people ADD COLUMN language TEXT;
