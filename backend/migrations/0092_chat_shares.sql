-- A read-only link to one reply or a whole chat. Its token is the link; `anyone` lets people who
-- aren't signed in open it; `mask` hides amounts, email addresses and phone numbers in what it
-- shows. It stops working once revoked or past `expires_at` (never, when that's null).
CREATE TABLE chat_shares (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token       TEXT NOT NULL UNIQUE,
    session_id  UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    created_by  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scope       TEXT NOT NULL CHECK (scope IN ('reply', 'chat')),
    message_id  UUID REFERENCES messages(id) ON DELETE CASCADE,
    mask        BOOLEAN NOT NULL DEFAULT true,
    anyone      BOOLEAN NOT NULL DEFAULT true,
    expires_at  TIMESTAMPTZ,
    revoked_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (scope = 'chat' OR message_id IS NOT NULL)
);
CREATE INDEX chat_shares_session_id_idx ON chat_shares (session_id);
