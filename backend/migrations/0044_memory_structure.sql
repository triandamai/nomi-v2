-- Memories become structured: what kind of thing each one is, where it came from, when it was
-- last useful, and whether a newer memory replaced it.
ALTER TABLE memory_items
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'fact'
        CHECK (kind IN ('preference', 'person', 'routine', 'goal', 'fact')),
    ADD COLUMN source_message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    ADD COLUMN last_used_at TIMESTAMPTZ,
    ADD COLUMN confirmed_at TIMESTAMPTZ,
    -- A memory that a newer one replaced ("moved to Bandung" over "lives in Jakarta"), or that
    -- faded or was merged away, is kept out of recall but not deleted.
    ADD COLUMN superseded_by UUID REFERENCES memory_items(id) ON DELETE SET NULL,
    ADD COLUMN archived_at TIMESTAMPTZ;

CREATE INDEX memory_items_live ON memory_items (user_id) WHERE archived_at IS NULL;

-- Usage rows go with the memory or the message they point at.
ALTER TABLE message_memory_usage
    DROP CONSTRAINT message_memory_usage_message_id_fkey,
    DROP CONSTRAINT message_memory_usage_memory_id_fkey,
    ADD CONSTRAINT message_memory_usage_message_id_fkey FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    ADD CONSTRAINT message_memory_usage_memory_id_fkey FOREIGN KEY (memory_id) REFERENCES memory_items(id) ON DELETE CASCADE;

-- Why a reply got a thumbs-down, when the person says.
ALTER TABLE message_feedback
    ADD COLUMN reason TEXT CHECK (reason IN ('wrong_memory', 'not_relevant', 'too_long', 'other'));

-- Working memory: a running summary of the older part of each chat, covering every message up to
-- and including `summary_through`.
ALTER TABLE sessions
    ADD COLUMN summary TEXT,
    ADD COLUMN summary_through TIMESTAMPTZ;

-- When each person's memories were last tidied (merged, faded, archived).
CREATE TABLE memory_upkeep (
    user_id         UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    consolidated_at TIMESTAMPTZ NOT NULL
);

-- Reading a chat's messages in order (history, and the summary sweep).
CREATE INDEX IF NOT EXISTS messages_session_created ON messages (session_id, created_at);
