-- Nomi's own look at a live app's address every few minutes: whether it answered.
CREATE TABLE site_uptime_checks (
    id          BIGSERIAL PRIMARY KEY,
    project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    up          BOOLEAN NOT NULL,
    -- The HTTP status it answered with; NULL when it didn't answer at all.
    status      INTEGER,
    checked_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX site_uptime_checks_project_at ON site_uptime_checks (project_id, checked_at);
