-- The version people see: one per crew reply that changed the files (and one per change made
-- outside a reply — the code editor, an import). `files_version` stays the counter the live
-- preview syncs on, which moves with every single file change.
ALTER TABLE projects ADD COLUMN version BIGINT NOT NULL DEFAULT 0;
-- The reply the current version belongs to (nomi_agent_core::reply); its next change stays in it.
ALTER TABLE projects ADD COLUMN version_reply_id UUID;
-- Existing projects carry on from the number they show now.
UPDATE projects SET version = files_version;

-- The version a publish shipped. Older publishes have none and show their files_version.
ALTER TABLE project_publishes ADD COLUMN version BIGINT;
