-- The design brief Rena saves before a site or app is designed (nomi_agent_core::design_brief):
-- structured, with every entry marked by where it came from. Each save is a new version of the
-- chat's brief; a version is never updated.
CREATE TABLE design_briefs (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id        UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    agent_session_id  UUID NOT NULL,
    user_id           UUID NOT NULL REFERENCES users(id),
    version           INT NOT NULL,
    content           JSONB NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (session_id, version)
);
