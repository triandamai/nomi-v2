CREATE TABLE scheduled_jobs (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id               UUID NOT NULL REFERENCES sessions(id),
    user_id                  UUID NOT NULL REFERENCES users(id),
    created_by_agent_type    TEXT NOT NULL,
    target_agent_type        TEXT NOT NULL,
    label                    TEXT NOT NULL,
    prompt                   TEXT NOT NULL,
    run_at                   TIMESTAMPTZ NOT NULL,
    recurrence               TEXT CHECK (recurrence IN ('daily', 'weekly', 'monthly')),
    recurrence_weekday       SMALLINT CHECK (recurrence_weekday BETWEEN 0 AND 6),
    recurrence_day_of_month  SMALLINT CHECK (recurrence_day_of_month BETWEEN 1 AND 31),
    status                   TEXT NOT NULL DEFAULT 'active'
                                 CHECK (status IN ('active', 'cancelled', 'completed')),
    claimed_at               TIMESTAMPTZ,
    last_fired_at            TIMESTAMPTZ,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    cancelled_at             TIMESTAMPTZ
);

CREATE INDEX scheduled_jobs_due_idx ON scheduled_jobs (run_at)
    WHERE status = 'active' AND claimed_at IS NULL;

ALTER TABLE user_preferences ADD COLUMN timezone TEXT NOT NULL DEFAULT 'UTC';
