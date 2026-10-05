-- The Workspace agent's own tables (see docs/agents/storage.md). Each person connects their
-- own Google account; nothing here is shared between users.

-- One Google connection per Nomi user. Tokens are encrypted with SETTINGS_ENCRYPTION_KEY.
CREATE TABLE workspace_connections (
    user_id                  UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    google_email             TEXT NOT NULL,
    -- What the user allowed: any of gmail, sheets, docs, drive, calendar.
    services                 TEXT[] NOT NULL,
    access_token_encrypted   BYTEA NOT NULL,
    refresh_token_encrypted  BYTEA,
    expires_at               TIMESTAMPTZ NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- An authorization in flight: the `state` sent to Google, tied to the user who started it so
-- the callback can only ever connect that user's account. Used once, valid for 10 minutes.
CREATE TABLE workspace_oauth_states (
    state              TEXT PRIMARY KEY,
    user_id            UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    services           TEXT[] NOT NULL,
    -- The chat to carry on in once connected (the request that asked for Google).
    resume_session_id  UUID REFERENCES sessions(id) ON DELETE SET NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- What the Workspace agent changed in a user's account, newest first on the Connections page.
CREATE TABLE workspace_activity (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    service     TEXT NOT NULL,
    summary     TEXT NOT NULL,
    link        TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX workspace_activity_user_idx ON workspace_activity (user_id, created_at DESC);
