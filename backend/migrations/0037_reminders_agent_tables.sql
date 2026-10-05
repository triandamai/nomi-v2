-- The Reminders agent's own table (agent-owned storage: see docs/agents/storage.md). Only
-- nomi-agent-reminders and the Reminders page's routes write it. Agent *tasks* scheduled for
-- later ("every Monday, summarize my spending") stay in the core scheduled_jobs table.
CREATE TABLE reminders (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- The chat it reports into.
    session_id               UUID NOT NULL REFERENCES sessions(id),
    title                    TEXT NOT NULL,
    notes                    TEXT,
    due_at                   TIMESTAMPTZ NOT NULL,
    recurrence               TEXT CHECK (recurrence IN ('daily', 'weekly', 'monthly')),
    recurrence_weekday       SMALLINT CHECK (recurrence_weekday BETWEEN 0 AND 6),
    recurrence_day_of_month  SMALLINT CHECK (recurrence_day_of_month BETWEEN 1 AND 31),
    -- active: waiting to fire. fired: a one-off that has gone off. done: ticked off.
    status                   TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'fired', 'done', 'cancelled')),
    created_by               TEXT NOT NULL DEFAULT 'agent' CHECK (created_by IN ('user', 'agent')),
    claimed_at               TIMESTAMPTZ,
    last_fired_at            TIMESTAMPTZ,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX reminders_due_idx ON reminders (due_at) WHERE status = 'active' AND claimed_at IS NULL;
CREATE INDEX reminders_user_idx ON reminders (user_id, status, due_at);
