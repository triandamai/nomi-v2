-- What an agent may do on an organization's projects, decided by the people who run them: one
-- row per permission, readable as `<agent>.<org>.<project>.<permission>` with its scope (allow or
-- denied). The inspector reads a published app blind (counts, columns, error totals) until a
-- project admin allows it to see what's inside (`read_data`, `read_logs`).
--
-- Kept as columns rather than only the dotted string, so a grant goes with its project or
-- organization when they're deleted and lookups use an index; `key` is the same thing as one
-- string, for logs and the settings sheet. A NULL project covers every project in the org.
CREATE TABLE agent_permissions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_type  TEXT NOT NULL CHECK (agent_type ~ '^[a-z][a-z0-9_]*$'),
    org_id      UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    project_id  UUID REFERENCES projects(id) ON DELETE CASCADE,
    permission  TEXT NOT NULL CHECK (permission ~ '^[a-z][a-z0-9_]*$'),
    scope       TEXT NOT NULL CHECK (scope IN ('allow', 'denied')),
    key         TEXT GENERATED ALWAYS AS (
                    agent_type || '.' || org_id::text || '.' || COALESCE(project_id::text, '*') || '.' || permission
                ) STORED,
    decided_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX agent_permissions_key_idx ON agent_permissions (key);
CREATE INDEX agent_permissions_project_idx ON agent_permissions (project_id) WHERE project_id IS NOT NULL;
