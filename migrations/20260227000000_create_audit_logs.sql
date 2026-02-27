CREATE TABLE IF NOT EXISTS audit_logs (
    id                BIGSERIAL    PRIMARY KEY,
    resource_type     TEXT         NOT NULL,
    resource_id       TEXT         NOT NULL,
    action            TEXT         NOT NULL,
    before_data       JSONB,
    after_data        JSONB,
    actor_discord_id  TEXT,
    actor_username    TEXT         NOT NULL DEFAULT 'unknown',
    actor_global_name TEXT,
    actor_avatar_url  TEXT,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource   ON audit_logs (resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs (created_at DESC);
