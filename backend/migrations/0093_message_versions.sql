-- Running a turn again: editing the person's message, or asking for the crew's reply again. Each
-- earlier take is kept as a version of the person's message: what they wrote then, and the
-- replies it got. `edited_at` marks a message the person changed.
ALTER TABLE messages ADD COLUMN edited_at TIMESTAMPTZ;

CREATE TABLE message_versions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id  UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    version     INTEGER NOT NULL CHECK (version > 0),
    text        TEXT NOT NULL,
    replies     JSONB NOT NULL DEFAULT '[]',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (message_id, version)
);
