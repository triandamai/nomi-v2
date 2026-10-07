-- Email sign-in codes: a password sign-in (and a new account) is finished with a 6-digit code
-- sent to the account's email.
ALTER TABLE web_credentials ADD COLUMN email_verified_at TIMESTAMPTZ;
-- Accounts made before codes existed count as confirmed (their next sign-in still needs a code).
UPDATE web_credentials SET email_verified_at = created_at WHERE email_verified_at IS NULL;

CREATE TABLE sign_in_challenges (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- 'register' (a new account confirming its email) or 'login'.
    purpose      TEXT NOT NULL CHECK (purpose IN ('register', 'login')),
    code_hash    TEXT NOT NULL,
    expires_at   TIMESTAMPTZ NOT NULL,
    attempts     INT NOT NULL DEFAULT 0,
    sends        INT NOT NULL DEFAULT 1,
    last_sent_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    used_at      TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX sign_in_challenges_user_recent ON sign_in_challenges (user_id, created_at DESC);
