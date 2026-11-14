-- Workspaces: who made one, closing one, declining an invite, invite links, where Nomi opens.

-- Who made the workspace ("created by Trian · 14 Jul 2026"). Earlier ones take their first owner.
ALTER TABLE organizations ADD COLUMN created_by UUID REFERENCES users(id) ON DELETE SET NULL;
UPDATE organizations o SET created_by = (
    SELECT m.user_id FROM memberships m WHERE m.org_id = o.id AND m.role = 'owner' ORDER BY m.created_at LIMIT 1
);

-- A deleted workspace (`DELETE /api/orgs/:id`) is kept for its old chats' sake; nobody is in it.
ALTER TABLE organizations ADD COLUMN deleted_at TIMESTAMPTZ;

-- The person an invite went to said no (`POST /api/invites/:code/decline`).
ALTER TABLE org_invites ADD COLUMN declined_at TIMESTAMPTZ;

-- A link anyone can join with (`POST /api/orgs/:id/invite-links`): it has no address and isn't
-- used up by the first person who joins; it works until it expires or is taken back.
ALTER TABLE org_invites ADD COLUMN is_link BOOLEAN NOT NULL DEFAULT false;

-- Where Nomi opens on sign-in (`PUT /api/preferences/open-org`): NULL is the workspace last used.
ALTER TABLE user_preferences ADD COLUMN open_org_id UUID REFERENCES organizations(id) ON DELETE SET NULL;

-- Open invites to an address, for `GET /api/invites`.
CREATE INDEX org_invites_open_email_idx ON org_invites (lower(email)) WHERE used_at IS NULL AND declined_at IS NULL;
