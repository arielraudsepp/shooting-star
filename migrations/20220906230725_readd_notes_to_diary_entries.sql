
UPDATE diary_entries SET notes = '';
ALTER TABLE diary_entries ALTER COLUMN notes SET DEFAULT '';
