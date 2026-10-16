-- Admin access is per area (models, support, integrations, ...) at one of three levels:
-- view, write (change things) or manage (everything, including what can't be undone).

ALTER TABLE user_permissions DROP CONSTRAINT IF EXISTS user_permissions_actions_check;
ALTER TABLE user_permissions ADD CONSTRAINT user_permissions_actions_check
    CHECK (actions <@ ARRAY['view', 'write', 'manage']::TEXT[] AND array_length(actions, 1) > 0);

-- `system_config` used to cover every area but users; it becomes the same access to each of them.
INSERT INTO user_permissions (user_id, scope_type, org_id, resource, actions, granted_by, created_at)
SELECT up.user_id, 'admin', NULL, area.resource,
       CASE WHEN 'manage' = ANY (up.actions) THEN ARRAY['view', 'write', 'manage'] ELSE ARRAY['view'] END,
       up.granted_by, up.created_at
FROM user_permissions up
CROSS JOIN (VALUES ('dashboard'), ('models'), ('tools'), ('agents'), ('support'), ('maintenance'), ('integrations')) AS area(resource)
WHERE up.scope_type = 'admin' AND up.resource = 'system_config'
  AND NOT EXISTS (
      SELECT 1 FROM user_permissions x
      WHERE x.user_id = up.user_id AND x.scope_type = 'admin' AND x.resource = area.resource
  );

-- Plans and broadcasts were part of user access.
INSERT INTO user_permissions (user_id, scope_type, org_id, resource, actions, granted_by, created_at)
SELECT up.user_id, 'admin', NULL, area.resource, up.actions, up.granted_by, up.created_at
FROM user_permissions up
CROSS JOIN (VALUES ('plans'), ('notifications')) AS area(resource)
WHERE up.scope_type = 'admin' AND up.resource = 'user'
  AND NOT EXISTS (
      SELECT 1 FROM user_permissions x
      WHERE x.user_id = up.user_id AND x.scope_type = 'admin' AND x.resource = area.resource
  );

DELETE FROM user_permissions WHERE scope_type = 'admin' AND resource = 'system_config';

-- Manage on users now includes write.
UPDATE user_permissions SET actions = ARRAY['view', 'write', 'manage']
WHERE scope_type = 'admin' AND 'manage' = ANY (actions) AND NOT ('write' = ANY (actions));

-- Where Nomi got to in each organization's Shipyard event feed, for catching up on events a
-- webhook didn't deliver.
CREATE TABLE shipyard_event_cursors (
    org_id      UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    after_id    TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
