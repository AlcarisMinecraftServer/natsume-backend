ALTER TABLE recipes RENAME COLUMN inputs TO materials;
ALTER TABLE recipes RENAME COLUMN output TO product;

ALTER TABLE recipes ALTER COLUMN cooldown DROP NOT NULL;
ALTER TABLE recipes ALTER COLUMN cooldown SET DEFAULT NULL;
ALTER TABLE recipes ALTER COLUMN unlock_level DROP NOT NULL;
ALTER TABLE recipes ALTER COLUMN unlock_level SET DEFAULT NULL;

CREATE TABLE IF NOT EXISTS recipe_audit_logs (
    id BIGSERIAL PRIMARY KEY,
    action TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    actor_discord_id TEXT,
    actor_username TEXT NOT NULL DEFAULT 'unknown',
    actor_global_name TEXT,
    actor_avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_recipe_audit_logs_recipe_id ON recipe_audit_logs (recipe_id);
CREATE INDEX IF NOT EXISTS idx_recipe_audit_logs_created_at ON recipe_audit_logs (created_at);
