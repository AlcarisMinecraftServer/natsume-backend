-- Legacy schema stored `custom_model_data` as INTEGER.
-- A later migration converted the column to JSONB via `to_jsonb(custom_model_data)`,
-- which turns legacy values into JSON numbers (e.g. `0`).
-- The application expects the new adjacently-tagged object form; clear legacy values.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'items'
          AND column_name = 'custom_model_data'
          AND data_type = 'jsonb'
    ) THEN
        UPDATE items
        SET custom_model_data = NULL
        WHERE custom_model_data IS NOT NULL
          AND jsonb_typeof(custom_model_data) = 'number';
    END IF;
END $$;
