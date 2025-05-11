-- Add migration script here
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name='items' AND column_name='tags'
    ) THEN
        ALTER TABLE items ADD COLUMN tags JSONB DEFAULT '[]'::jsonb;
    END IF;
END
$$;