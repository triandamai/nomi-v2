CREATE TABLE user_permissions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scope_type  TEXT NOT NULL CHECK (scope_type IN ('admin', 'org')),
    org_id      UUID REFERENCES organizations(id) ON DELETE CASCADE,
    resource    TEXT NOT NULL CHECK (resource ~ '^[a-z][a-z0-9_]*$'),
    actions     TEXT[] NOT NULL CHECK (actions <@ ARRAY['view', 'manage']::TEXT[] AND array_length(actions, 1) > 0),
    granted_by  UUID REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((scope_type = 'admin' AND org_id IS NULL) OR (scope_type = 'org' AND org_id IS NOT NULL))
);

CREATE INDEX user_permissions_user_id_idx ON user_permissions(user_id);
