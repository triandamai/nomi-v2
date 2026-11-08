-- Projects can run in a server-side sandbox instead of the user's browser: which runtime a
-- project uses, the sandbox each one has and what it should be doing, and a log of how long
-- each sandbox ran.

-- Projects made before this run in the browser.
ALTER TABLE projects ADD COLUMN runtime TEXT NOT NULL DEFAULT 'browser'
    CHECK (runtime IN ('browser', 'sandbox'));

-- One sandbox per project. `desired` is what the project wants, `state` is what the provider
-- last reported; a background worker moves state toward desired.
CREATE TABLE project_sandboxes (
    project_id     UUID PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    provider       TEXT NOT NULL,
    -- The provider's id for the sandbox, once it exists.
    sandbox_id     TEXT,
    desired        TEXT NOT NULL DEFAULT 'stopped'
                   CHECK (desired IN ('running', 'stopped')),
    state          TEXT NOT NULL DEFAULT 'none'
                   CHECK (state IN ('none', 'creating', 'starting', 'running', 'stopping', 'stopped', 'error', 'gone')),
    -- The files_version last copied into the sandbox.
    synced_version BIGINT NOT NULL DEFAULT 0,
    -- What the sandbox holds: path -> updated_at of each file as last copied in. Emptied with synced_version.
    synced_manifest JSONB NOT NULL DEFAULT '{}'::jsonb,
    preview_url    TEXT,
    -- The dependency files (package.json and the lockfile, with their versions) `npm install` last ran for. Emptied with synced_version.
    deps_hash      TEXT,
    -- Failed attempts since the last settled state, when to try next, and why the last one failed.
    retry_at       TIMESTAMPTZ,
    -- Held while a worker walks the row, so no second worker takes it; separate from retry_at,
    -- which a person's open or close may clear.
    leased_until   TIMESTAMPTZ,
    attempts       INT NOT NULL DEFAULT 0,
    last_error     TEXT,
    -- The last sign anyone uses the sandbox: the sandbox page's heartbeat or Koda waking it. The
    -- idle sweep stops a sandbox once this (and the project's last command) is old enough.
    last_active_at TIMESTAMPTZ,
    -- When the worker last asked the provider whether this running sandbox is still up.
    alive_checked_at TIMESTAMPTZ,
    -- The provider's id of the dev server process started last, so the next start can kill it.
    devserver_process TEXT,
    -- The end of the dev server's output (bounded by the worker), readable from any process.
    log_tail       TEXT NOT NULL DEFAULT '',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The worker looks for rows whose state hasn't reached what they want.
CREATE INDEX project_sandboxes_due_idx ON project_sandboxes (desired, state, retry_at);

-- One row per stretch a sandbox ran, for metering.
CREATE TABLE sandbox_sessions (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    size       TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    stopped_at TIMESTAMPTZ
);

CREATE INDEX sandbox_sessions_project_idx ON sandbox_sessions (project_id, started_at);
-- The monthly allowance sums one person's sessions.
CREATE INDEX sandbox_sessions_user_idx ON sandbox_sessions (user_id, started_at);

-- A project has at most one open stretch at a time.
CREATE UNIQUE INDEX sandbox_sessions_one_open_idx ON sandbox_sessions (project_id) WHERE stopped_at IS NULL;

-- Sandboxes that still have to be deleted at the provider. A project's sandbox row cascades away
-- with the project (or its owner), so the trigger below copies its sandbox id here in the same
-- transaction, whichever path deleted it. There is deliberately no foreign key to projects. The
-- sandbox worker drains this table; rows wait while the sandbox runtime is switched off.
CREATE TABLE sandbox_deletions (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider   TEXT NOT NULL,
    sandbox_id TEXT NOT NULL UNIQUE,
    project_id UUID,
    attempts   INT NOT NULL DEFAULT 0,
    retry_at   TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX sandbox_deletions_due_idx ON sandbox_deletions (retry_at);

CREATE FUNCTION enqueue_sandbox_deletion() RETURNS trigger AS $$
BEGIN
    IF OLD.sandbox_id IS NOT NULL THEN
        INSERT INTO sandbox_deletions (provider, sandbox_id, project_id)
        VALUES (OLD.provider, OLD.sandbox_id, OLD.project_id)
        ON CONFLICT (sandbox_id) DO NOTHING;
    END IF;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER project_sandboxes_enqueue_deletion
    BEFORE DELETE ON project_sandboxes
    FOR EACH ROW EXECUTE FUNCTION enqueue_sandbox_deletion();
