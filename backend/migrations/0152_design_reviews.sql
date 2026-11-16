-- Checks of what was built against the approved design (nomi_agent_core::design_review): the
-- handoff's colours, fonts and element ids looked for in the project's files. Each check is a
-- new version; none is ever changed.
CREATE TABLE design_reviews (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id        UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    agent_session_id  UUID NOT NULL,
    user_id           UUID NOT NULL REFERENCES users(id),
    version           INT NOT NULL,
    -- The handoff (design_handoffs.version of the same chat) the build was checked against.
    handoff_version   INT NOT NULL,
    project_id        UUID NOT NULL,
    review            JSONB NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (session_id, version)
);
