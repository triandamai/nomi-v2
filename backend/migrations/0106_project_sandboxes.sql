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
    preview_url    TEXT,
    -- Failed attempts since the last settled state, when to try next, and why the last one failed.
    retry_at       TIMESTAMPTZ,
    attempts       INT NOT NULL DEFAULT 0,
    last_error     TEXT,
    last_active_at TIMESTAMPTZ,
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
