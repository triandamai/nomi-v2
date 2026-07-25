CREATE TABLE sessions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      UUID NOT NULL REFERENCES organizations(id),
    channel     TEXT NOT NULL,
    chat_type   TEXT NOT NULL DEFAULT 'dm',
    chat_id     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (channel, chat_id)
);

CREATE TABLE messages (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id                  UUID NOT NULL REFERENCES sessions(id),
    sender_channel_identity_id  UUID REFERENCES channel_identities(id),
    content                     TEXT NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE session_participants (
    session_id  UUID NOT NULL REFERENCES sessions(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    invited_by  UUID REFERENCES users(id),
    added_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, user_id)
);
