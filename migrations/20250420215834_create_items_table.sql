-- Add migration script here
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY,
    version BIGINT NOT NULL,
    name TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('food', 'tool', 'armor')),
    lore JSONB NOT NULL,
    rarity SMALLINT NOT NULL,
    max_stack SMALLINT NOT NULL,
    custom_model_data INTEGER NOT NULL,
    price JSONB NOT NULL,
    data JSONB NOT NULL
);
