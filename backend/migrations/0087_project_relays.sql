-- Each project's own Relay: one per environment, running on Shipyard next to its own database
-- (see docs/superpowers/specs/2026-10-27-relay-on-shipyard-design.md). A project with no row here
-- uses the shared Relay saved in the admin, as every project did before.
CREATE TABLE project_relays (
    project_id   UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    env          TEXT NOT NULL CHECK (env IN ('dev', 'production')),
    -- The environment's public address, once Shipyard has given it.
    url          TEXT,
    -- What Nomi calls this Relay with, sealed with SETTINGS_ENCRYPTION_KEY.
    control_key  BYTEA NOT NULL,
    version      TEXT NOT NULL,
    state        TEXT NOT NULL CHECK (state IN ('creating', 'ready', 'sleeping', 'waking', 'failed')),
    -- The setup step it is on, or failed at.
    step         TEXT,
    error        TEXT,
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, env)
);
