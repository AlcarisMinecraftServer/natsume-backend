-- Add migration script here
ALTER TABLE items ADD COLUMN tags JSONB DEFAULT '[]'::jsonb;
