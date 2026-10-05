-- Sign in with Google. A Google account signs in to exactly one Nomi user; a user can sign in
-- with a password, Google, or both. (Connecting Google Workspace for the Workspace agent is
-- separate: see workspace_connections.)

CREATE TABLE google_identities (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    -- Google's stable account id (the ID token's `sub`), never the email, which can change.
    google_sub  TEXT NOT NULL UNIQUE,
    email       TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Whether the user can also sign in with a password. Accounts created with Google start
-- without one (their stored hash is of a random secret nobody knows).
ALTER TABLE web_credentials ADD COLUMN password_login BOOLEAN NOT NULL DEFAULT true;

-- A Google sign-in in flight. `purpose` 'sign_in' has no user yet; 'link' adds Google to the
-- signed-in user. Used once, valid for 10 minutes.
CREATE TABLE google_sign_in_states (
    state        TEXT PRIMARY KEY,
    purpose      TEXT NOT NULL CHECK (purpose IN ('sign_in', 'link')),
    user_id      UUID REFERENCES users(id) ON DELETE CASCADE,
    invite_code  TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
