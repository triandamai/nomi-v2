-- The design directions Sora proposes from a chat's brief (nomi_agent_core::design_directions):
-- two to four per set, structured. Each save is a new version of the chat's set; a version is
-- never updated. `brief_version` is the design brief the set was made from, when there was one.
CREATE TABLE design_direction_sets (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id        UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    agent_session_id  UUID NOT NULL,
    user_id           UUID NOT NULL REFERENCES users(id),
    brief_version     INT,
    version           INT NOT NULL,
    content           JSONB NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (session_id, version)
);
