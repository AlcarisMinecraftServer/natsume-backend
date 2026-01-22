CREATE TABLE IF NOT EXISTS file_uploads (
    upload_id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    key TEXT NOT NULL,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size BIGINT NOT NULL,
    part_size BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'in_progress' CHECK (status IN ('in_progress', 'completed', 'aborted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS file_upload_parts (
    upload_id TEXT NOT NULL REFERENCES file_uploads(upload_id) ON DELETE CASCADE,
    part_number INTEGER NOT NULL,
    etag TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (upload_id, part_number)
);

CREATE INDEX IF NOT EXISTS idx_file_uploads_user_id ON file_uploads (user_id);
CREATE INDEX IF NOT EXISTS idx_file_upload_parts_upload_id ON file_upload_parts (upload_id);
