CREATE TABLE background_tasks (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id          UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_type          TEXT NOT NULL,
    label               TEXT NOT NULL,
    command             TEXT NOT NULL,
    project_id          UUID,
    timeout_seconds     INTEGER NOT NULL DEFAULT 180 CHECK (timeout_seconds >= 1 AND timeout_seconds <= 600),
    status              TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    exit_code           INTEGER,
    output              TEXT,
    error               TEXT,
    claimed_at          TIMESTAMPTZ,
    completed_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX background_tasks_pending_idx ON background_tasks (created_at) WHERE status = 'pending';
CREATE INDEX background_tasks_session_idx ON background_tasks (session_id, created_at DESC);
