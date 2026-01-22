-- Add migration script here
ALTER TABLE items 
    ADD COLUMN IF NOT EXISTS item_model TEXT,
    ADD COLUMN IF NOT EXISTS tooltip_style TEXT;

ALTER TABLE items
    ALTER COLUMN custom_model_data TYPE JSONB
    USING to_jsonb(custom_model_data);

ALTER TABLE items
    ALTER COLUMN custom_model_data DROP NOT NULL,
    ALTER COLUMN custom_model_data SET DEFAULT NULL;