-- Files the crew makes in a chat (Maya's make_file) live in the same table and blob store as
-- files people attach. `made_by` names the agent that made one (null for an upload),
-- `made_summary` is the line shown with it and re-used when it is made again, and `made_from`
-- the attachments it was made out of. A made file the person deletes keeps its row with
-- `deleted_at` set (its bytes go, and it stops counting against their storage).
ALTER TABLE attachments
    ADD COLUMN made_by TEXT,
    ADD COLUMN made_summary TEXT,
    ADD COLUMN made_from UUID[] NOT NULL DEFAULT '{}',
    ADD COLUMN deleted_at TIMESTAMPTZ;

CREATE INDEX attachments_made_in_chat ON attachments (session_id, created_at DESC) WHERE made_by IS NOT NULL;
