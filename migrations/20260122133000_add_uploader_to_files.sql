ALTER TABLE files ADD COLUMN IF NOT EXISTS uploader_username TEXT;
ALTER TABLE files ADD COLUMN IF NOT EXISTS uploader_global_name TEXT;
ALTER TABLE files ADD COLUMN IF NOT EXISTS uploader_avatar_url TEXT;
