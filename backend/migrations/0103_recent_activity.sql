-- What each person did lately, across all their chats ("Today: Dana logged lunch, Rp 50k"):
-- a short note the memory worker rewrites when a chat goes quiet, and every agent reads.
-- Unlike memory_items it is not lasting facts: it's replaced, never recalled by similarity.
ALTER TABLE users
    ADD COLUMN recent_activity    TEXT,
    ADD COLUMN recent_activity_at TIMESTAMPTZ;

-- The newest message of a chat already folded into its person's note.
ALTER TABLE sessions ADD COLUMN activity_noted_through TIMESTAMPTZ;
