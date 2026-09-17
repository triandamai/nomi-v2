-- backend/migrations/0024_agent_plans.sql

CREATE TABLE agent_plans (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id        UUID NOT NULL REFERENCES sessions(id),
    agent_session_id  UUID NOT NULL,
    user_id           UUID NOT NULL REFERENCES users(id),
    title             TEXT NOT NULL,
    content           TEXT,
    content_s3_key    TEXT,
    version           INT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((content IS NOT NULL) <> (content_s3_key IS NOT NULL))
);

CREATE INDEX agent_plans_lookup_idx ON agent_plans (agent_session_id, version);
