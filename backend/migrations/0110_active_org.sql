-- The workspace a person last switched to (`PUT /api/auth/active-org`). Sign-in and refresh put
-- it in the access token while they're still an active member; otherwise (and when NULL) their
-- first workspace, as before.
ALTER TABLE users ADD COLUMN active_org_id UUID REFERENCES organizations(id) ON DELETE SET NULL;
