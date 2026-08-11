CREATE TABLE turn_jobs (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id                  UUID NOT NULL REFERENCES sessions(id),
    sender_channel_identity_id  UUID NOT NULL,
    text                        TEXT NOT NULL,
    org_id_hint                 UUID,
    status                      TEXT NOT NULL DEFAULT 'pending',
    claimed_at                  TIMESTAMPTZ,
    completed_at                TIMESTAMPTZ,
    error                       TEXT,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX turn_jobs_pending_idx ON turn_jobs (created_at) WHERE status = 'pending';
