-- Learning from a chat happens in the background (memory worker), never on the reply path:
-- after an agent answers, the turn only queues what to learn from.
CREATE TABLE memory_jobs (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- The person's message this is about (its memory's source), when it's in the chat.
    source_message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    person_text       TEXT NOT NULL,
    answer            TEXT NOT NULL,
    attempts          INT NOT NULL DEFAULT 0,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX memory_jobs_created ON memory_jobs (created_at);
