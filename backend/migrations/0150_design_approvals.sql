-- The person's approval of one version of a chat's design previews
-- (nomi_agent_core::design_approval), given in the Design Studio. Rows are only ever added: the
-- newest is the approval that stands, the earlier ones are the record of what was approved when.
CREATE TABLE design_approvals (
    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    session_id        UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id           UUID NOT NULL REFERENCES users(id),
    -- The approved preview version (design_artifacts.version of the same chat).
    artifact_version  INT NOT NULL,
    note              TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX design_approvals_session_idx ON design_approvals (session_id, created_at DESC, id DESC);
