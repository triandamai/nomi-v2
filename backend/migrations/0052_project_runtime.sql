-- Projects run in the browser (WebContainer): which stack Koda builds on, a counter the open
-- project page watches to pick up changed files, the result of the last install/build the page
-- ran, and the commands Koda asks the page to run.

-- Projects made before this were plain static pages.
ALTER TABLE projects ADD COLUMN stack TEXT NOT NULL DEFAULT 'static'
    CHECK (stack IN ('sveltekit', 'svelte', 'react', 'vue', 'astro', 'static'));
ALTER TABLE projects ALTER COLUMN stack SET DEFAULT 'sveltekit';
-- A starter the project began from (reserved: none yet).
ALTER TABLE projects ADD COLUMN template TEXT;
-- Goes up on every file change.
ALTER TABLE projects ADD COLUMN files_version BIGINT NOT NULL DEFAULT 0;
-- When an open project page last checked in.
ALTER TABLE projects ADD COLUMN runner_seen_at TIMESTAMPTZ;
-- The last check (install, type check, build) the page ran, and on which version.
ALTER TABLE projects ADD COLUMN checked_version BIGINT;
ALTER TABLE projects ADD COLUMN check_ok BOOLEAN;
ALTER TABLE projects ADD COLUMN check_output TEXT;
-- The version whose failed check was already handed to Koda, so it's handed over once.
ALTER TABLE projects ADD COLUMN fix_requested_version BIGINT;

CREATE TABLE project_runs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    command     TEXT NOT NULL,
    args        JSONB NOT NULL DEFAULT '[]',
    status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'running', 'done', 'failed', 'timed_out')),
    exit_code   INTEGER,
    output      TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at  TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);

CREATE INDEX project_runs_pending_idx ON project_runs (project_id, created_at) WHERE status = 'pending';
