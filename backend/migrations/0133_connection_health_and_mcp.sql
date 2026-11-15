-- When a connection's sign-in lapsed (the app refused it on use): kept, so the person sees
-- "Needs you" with Reconnect instead of the connection quietly vanishing. Connecting again
-- clears it.
ALTER TABLE workspace_connections ADD COLUMN expired_at TIMESTAMPTZ, ADD COLUMN last_used_at TIMESTAMPTZ;
ALTER TABLE health_connections ADD COLUMN expired_at TIMESTAMPTZ;
ALTER TABLE github_connections ADD COLUMN expired_at TIMESTAMPTZ;

-- A person's own MCP servers. Only remote (http/https) servers are kept; Nomi never runs a
-- local command for anyone. Nothing in the crew calls them yet.
CREATE TABLE mcp_servers (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    url         TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, url)
);
CREATE INDEX mcp_servers_user ON mcp_servers (user_id, created_at);
