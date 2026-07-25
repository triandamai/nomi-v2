-- migrations/0005_agent_sessions.sql
CREATE TABLE agent_sessions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id                  UUID NOT NULL REFERENCES sessions(id),
    sender_channel_identity_id  UUID NOT NULL REFERENCES channel_identities(id),
    agent_type                  TEXT NOT NULL,
    status                      TEXT NOT NULL,
    state                       JSONB NOT NULL DEFAULT '{}',
    started_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_activity_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at                    TIMESTAMPTZ
);

CREATE UNIQUE INDEX agent_sessions_one_active_per_speaker
    ON agent_sessions (session_id, sender_channel_identity_id) WHERE status = 'active';
