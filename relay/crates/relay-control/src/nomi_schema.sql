-- The nomi schema: what Relay keeps for every project in a database, next to the project's own
-- tables. Every row has a project_id; row-level security keeps each project's roles to their
-- own rows, and a project's role names say which project they are (app_<id>_<kind>), which is
-- how nomi.project_id() knows. Safe to run again: every statement checks first.
CREATE SCHEMA IF NOT EXISTS nomi;

CREATE TABLE IF NOT EXISTS nomi.users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL,
    email           TEXT,
    email_verified  BOOLEAN NOT NULL DEFAULT false,
    user_metadata   JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_sign_in_at TIMESTAMPTZ,
    UNIQUE (project_id, email)
);
CREATE TABLE IF NOT EXISTS nomi.identities (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id       UUID NOT NULL,
    user_id          UUID NOT NULL REFERENCES nomi.users(id) ON DELETE CASCADE,
    provider         TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    email            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, provider, provider_user_id)
);
CREATE TABLE IF NOT EXISTS nomi.sessions (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id         UUID NOT NULL,
    user_id            UUID NOT NULL REFERENCES nomi.users(id) ON DELETE CASCADE,
    family             UUID NOT NULL,
    refresh_token_hash TEXT NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at         TIMESTAMPTZ NOT NULL,
    revoked_at         TIMESTAMPTZ
);
CREATE TABLE IF NOT EXISTS nomi.migrations (
    project_id UUID NOT NULL,
    name       TEXT NOT NULL,
    checksum   TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, name)
);
CREATE TABLE IF NOT EXISTS nomi.buckets (
    project_id UUID NOT NULL,
    name       TEXT NOT NULL,
    public     BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, name)
);

-- The request's claims, set by the gateway for each request (`request.jwt.claims`).
CREATE OR REPLACE FUNCTION nomi.claims() RETURNS jsonb LANGUAGE sql STABLE AS $$
    SELECT COALESCE(NULLIF(current_setting('request.jwt.claims', true), ''), '{}')::jsonb
$$;
-- The signed-in user, or null.
CREATE OR REPLACE FUNCTION nomi.uid() RETURNS uuid LANGUAGE sql STABLE AS $$
    SELECT NULLIF(nomi.claims()->>'sub', '')::uuid
$$;
-- anon, authenticated or service.
CREATE OR REPLACE FUNCTION nomi.role() RETURNS text LANGUAGE sql STABLE AS $$
    SELECT COALESCE(nomi.claims()->>'role', 'anon')
$$;
-- The project the current role belongs to, from its name; null for Relay itself.
CREATE OR REPLACE FUNCTION nomi.project_id() RETURNS uuid LANGUAGE sql STABLE AS $$
    SELECT CASE WHEN current_user ~ '^app_[0-9a-f]{32}_' THEN substring(current_user FROM 5 FOR 32)::uuid END
$$;

DO $$
DECLARE t text;
BEGIN
    FOREACH t IN ARRAY ARRAY['users', 'identities', 'sessions', 'migrations', 'buckets'] LOOP
        EXECUTE format('ALTER TABLE nomi.%I ENABLE ROW LEVEL SECURITY', t);
        IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE schemaname = 'nomi' AND tablename = t AND policyname = 'own project') THEN
            EXECUTE format('CREATE POLICY "own project" ON nomi.%I USING (project_id = nomi.project_id()) WITH CHECK (project_id = nomi.project_id())', t);
        END IF;
    END LOOP;
END $$;
