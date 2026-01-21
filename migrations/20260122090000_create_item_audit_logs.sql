CREATE TABLE IF NOT EXISTS item_audit_logs (
    id BIGSERIAL PRIMARY KEY,
    action TEXT NOT NULL,
    item_id TEXT NOT NULL,
    actor_discord_id TEXT,
    actor_username TEXT NOT NULL DEFAULT 'unknown',
    actor_global_name TEXT,
    actor_avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_item_audit_logs_item_id ON item_audit_logs (item_id);
CREATE INDEX IF NOT EXISTS idx_item_audit_logs_created_at ON item_audit_logs (created_at);
