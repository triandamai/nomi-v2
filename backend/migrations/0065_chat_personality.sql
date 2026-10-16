-- A chat can have its own personality. Without one it uses the person's own (user_personality),
-- which every new chat starts from.
CREATE TABLE session_personality (
    session_id   UUID PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    description  TEXT NOT NULL,
    set_by       UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX session_personality_set_by_idx ON session_personality (set_by);
