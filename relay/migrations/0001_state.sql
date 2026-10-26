-- Relay's own state: which projects have a backend, and for each environment where its
-- database is (null: the shared database for that environment) and the secret its API keys are
-- signed with. Secrets and dedicated database addresses are sealed with RELAY_ENCRYPTION_KEY.
CREATE SCHEMA IF NOT EXISTS relay;

CREATE TABLE relay.projects (
    id         UUID PRIMARY KEY,
    name       TEXT NOT NULL,
    slug       TEXT NOT NULL UNIQUE,
    tier       TEXT NOT NULL CHECK (tier IN ('free', 'dedicated')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE relay.environments (
    project_id            UUID NOT NULL REFERENCES relay.projects(id) ON DELETE CASCADE,
    env                   TEXT NOT NULL CHECK (env IN ('dev', 'production')),
    database_url_sealed   BYTEA,
    signing_secret_sealed BYTEA NOT NULL,
    -- When its API keys were issued (they're signed with this time, so asking again gives the
    -- same keys).
    keys_issued_at        BIGINT NOT NULL,
    PRIMARY KEY (project_id, env)
);
