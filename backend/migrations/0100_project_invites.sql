-- An invite to an organization can be an invite to build one of its projects together: who
-- accepts it also joins that project, with the role it was sent with (admin, member, viewer).
ALTER TABLE org_invites ADD COLUMN project_id UUID REFERENCES projects(id) ON DELETE SET NULL;
ALTER TABLE org_invites ADD COLUMN project_role TEXT CHECK (project_role IN ('admin', 'member', 'viewer'));
