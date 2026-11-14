-- A person's own crew: members they switched off, what each may do (allow / ask first / never),
-- each member's soul.md with its versions, and members of their own.

-- A member switched off for this person: the router and the crew pass it by. No row: on.
CREATE TABLE user_agent_settings (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_type  TEXT NOT NULL,
    enabled     BOOLEAN NOT NULL DEFAULT true,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, agent_type)
);

-- What a member may do for this person, by tool: allow (no card), ask (always the approval
-- card) or never (refused). No row: the member's own default.
CREATE TABLE user_agent_permissions (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_type  TEXT NOT NULL,
    key         TEXT NOT NULL,
    level       TEXT NOT NULL CHECK (level IN ('allow', 'ask', 'never')),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, agent_type, key)
);

-- A member's soul.md for this person, layered on its instructions: every save is a version, and
-- `current_version` says which one is in use (restoring an older one moves it back).
CREATE TABLE user_agent_souls (
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_type       TEXT NOT NULL,
    current_version  INT NOT NULL,
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, agent_type)
);
CREATE TABLE user_agent_soul_versions (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_type  TEXT NOT NULL,
    version     INT NOT NULL,
    text        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, agent_type, version)
);

-- A person's own member: a dynamic agent only they see and can talk to (NULL: an admin's, for
-- everyone), and the template it started from.
ALTER TABLE dynamic_agents
    ADD COLUMN owner_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN template_key TEXT;
CREATE INDEX dynamic_agents_owner ON dynamic_agents (owner_user_id) WHERE owner_user_id IS NOT NULL;
