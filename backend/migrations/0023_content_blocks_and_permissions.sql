-- backend/migrations/0023_content_blocks_and_permissions.sql

ALTER TABLE messages ADD COLUMN content_blocks JSONB;

CREATE TABLE tool_permission_rules (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id),
    tool_name    TEXT NOT NULL,
    path_pattern TEXT,
    decision     TEXT NOT NULL CHECK (decision IN ('allow', 'deny')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX tool_permission_rules_lookup_idx ON tool_permission_rules (user_id, tool_name);

-- Parallel queue to `turn_jobs`, for resuming a turn after an approval decision instead of
-- ingesting a new inbound message. Claimed by the same worker process (see worker.rs), on its
-- own NOTIFY channel so the existing turn_jobs claim loop is untouched.
CREATE TABLE approval_resumes (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id    UUID NOT NULL REFERENCES messages(id),
    decision      TEXT NOT NULL CHECK (decision IN ('approve', 'deny')),
    remember      BOOLEAN NOT NULL DEFAULT false,
    status        TEXT NOT NULL DEFAULT 'pending',
    claimed_at    TIMESTAMPTZ,
    completed_at  TIMESTAMPTZ,
    error         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX approval_resumes_pending_idx ON approval_resumes (created_at) WHERE status = 'pending';
