-- A person's GitHub, for pushing a project's code to a repository: through Nomi's OAuth app
-- (`oauth`) or a token they made themselves (`token`). The token is encrypted with the settings
-- key. `github_oauth_states` remembers who started connecting, for the ten minutes it may take.
CREATE TABLE github_connections (
    user_id          UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    login            TEXT NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('oauth', 'token')),
    token_encrypted  BYTEA NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at     TIMESTAMPTZ
);

CREATE TABLE github_oauth_states (
    state       TEXT PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
