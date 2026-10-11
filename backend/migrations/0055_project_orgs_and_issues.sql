-- Projects belong to an organization, and everyone active in it can open and work on them.
-- Existing projects go to the organization of the chat they were started in.
ALTER TABLE projects ADD COLUMN org_id UUID REFERENCES organizations(id);
UPDATE projects p SET org_id = s.org_id FROM sessions s WHERE s.id = p.session_id;
ALTER TABLE projects ALTER COLUMN org_id SET NOT NULL;
CREATE INDEX projects_org_id_idx ON projects (org_id);

-- The organization new projects go to, picked the first time the person opens Projects.
ALTER TABLE user_preferences ADD COLUMN project_org_id UUID REFERENCES organizations(id) ON DELETE SET NULL;

-- A project started without naming an organization (Rena turning a plain chat into a project)
-- goes to the one the person picked for projects, while they're still in it, else to its chat's.
CREATE FUNCTION projects_default_org() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.org_id IS NULL THEN
        NEW.org_id := (
            SELECT p.project_org_id FROM user_preferences p
            JOIN memberships m ON m.org_id = p.project_org_id AND m.user_id = p.user_id AND m.status = 'active'
            WHERE p.user_id = NEW.user_id
        );
    END IF;
    IF NEW.org_id IS NULL THEN
        NEW.org_id := (SELECT org_id FROM sessions WHERE id = NEW.session_id);
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER projects_default_org BEFORE INSERT ON projects FOR EACH ROW EXECUTE FUNCTION projects_default_org();

-- Whether `person` may open `project`: they started it, or they're an active member of its
-- organization.
CREATE FUNCTION can_open_project(project UUID, person UUID) RETURNS boolean LANGUAGE sql STABLE AS $$
    SELECT EXISTS (
        SELECT 1 FROM projects p
        WHERE p.id = project
          AND (p.user_id = person OR EXISTS (
              SELECT 1 FROM memberships m WHERE m.org_id = p.org_id AND m.user_id = person AND m.status = 'active'))
    )
$$;

-- Invites sent to an address: only that person can use them.
ALTER TABLE org_invites ADD COLUMN email TEXT;
ALTER TABLE org_invites ADD COLUMN used_by UUID REFERENCES users(id);
CREATE INDEX org_invites_org_id_idx ON org_invites (org_id) WHERE used_at IS NULL;

-- Feature requests, enhancements and bugs filed against a project, numbered per project.
CREATE TABLE project_issues (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    number      INTEGER NOT NULL,
    title       TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 200),
    body        TEXT NOT NULL DEFAULT '',
    kind        TEXT NOT NULL CHECK (kind IN ('feature', 'enhancement', 'bug')),
    severity    TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    status      TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    created_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    closed_at   TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, number)
);
CREATE INDEX project_issues_project_status_idx ON project_issues (project_id, status, number DESC);
