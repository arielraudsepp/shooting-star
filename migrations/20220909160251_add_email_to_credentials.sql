-- Add migration script here
ALTER TABLE users ADD COLUMN name TEXT DEFAULT '' NOT NULL;
ALTER TABLE users DROP COLUMN username;
ALTER TABLE users ADD COLUMN email TEXT DEFAULT '' NOT NULL;
