-- The Health agent's own tables (see docs/agents/storage.md). Each person connects their own
-- health and fitness apps; nothing here is shared between users.

-- One row per connected app. Hevy (workouts and routines) is the first; health monitors
-- (Samsung Health, Apple Health, ...) join later. Keys are encrypted with SETTINGS_ENCRYPTION_KEY.
CREATE TABLE health_connections (
    user_id            UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider           TEXT NOT NULL CHECK (provider IN ('hevy')),
    api_key_encrypted  BYTEA NOT NULL,
    -- The account it belongs to, as the app names it (shown on the setup page).
    account_name       TEXT,
    account_id         TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at       TIMESTAMPTZ,
    PRIMARY KEY (user_id, provider)
);
