-- The package Koda builds from (nomi_agent_core::design_handoff): what Sora wrote down about
-- an approved design, with the brief, the feedback's outcome and the approved preview's
-- version and address as they were then. Each handoff is a new version; none is ever changed.
CREATE TABLE design_handoffs (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id        UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    agent_session_id  UUID NOT NULL,
    user_id           UUID NOT NULL REFERENCES users(id),
    version           INT NOT NULL,
    -- The approved preview version it describes, and the brief version it came from.
    artifact_version  INT NOT NULL,
    brief_version     INT,
    package           JSONB NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (session_id, version)
);
