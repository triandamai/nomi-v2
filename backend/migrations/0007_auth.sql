ALTER TABLE users ADD COLUMN is_platform_admin BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE web_credentials (
    user_id       UUID PRIMARY KEY REFERENCES users(id),
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id),
    token_hash  TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL,
    revoked_at  TIMESTAMPTZ
);

ALTER TABLE memberships ADD CONSTRAINT memberships_role_check
    CHECK (role IN ('owner', 'admin', 'member'));
ALTER TABLE memberships ADD CONSTRAINT memberships_status_check
    CHECK (status IN ('invited', 'active', 'removed'));
ALTER TABLE org_invites ADD CONSTRAINT org_invites_role_check
    CHECK (role IN ('owner', 'admin', 'member'));
ALTER TABLE agent_sessions ADD CONSTRAINT agent_sessions_status_check
    CHECK (status IN ('active', 'completed', 'cancelled', 'expired'));
