CREATE TABLE agent_delegations (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id             UUID NOT NULL REFERENCES sessions(id),
    user_id                UUID NOT NULL REFERENCES users(id),
    requesting_agent_type  TEXT NOT NULL,
    target_agent_type      TEXT NOT NULL,
    task                   TEXT NOT NULL,
    status                 TEXT NOT NULL DEFAULT 'pending',
    result                 TEXT,
    error                  TEXT,
    claimed_at             TIMESTAMPTZ,
    completed_at           TIMESTAMPTZ,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX agent_delegations_pending_idx ON agent_delegations (created_at) WHERE status = 'pending';
CREATE INDEX agent_delegations_session_idx ON agent_delegations (session_id, created_at DESC);
