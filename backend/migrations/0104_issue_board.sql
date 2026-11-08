-- A project's issues as a board: columns people add, plus two fixed ones (Backlog, Ready to dev).
-- An issue moved into Ready to dev is queued for the crew; the issue worker gives it a chat of its
-- own and hands it to the crew member assigned to it.

CREATE TABLE project_issue_columns (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name       TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 40),
    position   INTEGER NOT NULL,
    -- 'backlog' or 'ready' for the two every project has; NULL for the ones people add.
    fixed      TEXT CHECK (fixed IN ('backlog', 'ready')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, fixed)
);
CREATE INDEX project_issue_columns_project_idx ON project_issue_columns (project_id, position);

-- Type is bug, task or enhancement now: a feature request is a task.
ALTER TABLE project_issues DROP CONSTRAINT project_issues_kind_check;
UPDATE project_issues SET kind = 'task' WHERE kind = 'feature';
ALTER TABLE project_issues ADD CONSTRAINT project_issues_kind_check CHECK (kind IN ('task', 'enhancement', 'bug'));

ALTER TABLE project_issues
    ADD COLUMN column_id UUID REFERENCES project_issue_columns(id) ON DELETE SET NULL,
    -- Order inside the column: a move takes the midpoint of its new neighbours.
    ADD COLUMN position DOUBLE PRECISION NOT NULL DEFAULT 0,
    ADD COLUMN start_on DATE,
    ADD COLUMN due_on DATE,
    ADD COLUMN points SMALLINT CHECK (points > 0),
    -- Who works it: the crew member it was assigned to, or whoever they handed it to.
    ADD COLUMN crew_agent TEXT NOT NULL DEFAULT 'coding',
    -- The crew member who handed it on, when it was handed on.
    ADD COLUMN assigned_by_agent TEXT,
    -- NULL until it reaches Ready to dev.
    ADD COLUMN crew_state TEXT CHECK (crew_state IN ('queued', 'working', 'done', 'failed')),
    ADD COLUMN queued_by UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN picked_up_at TIMESTAMPTZ,
    -- The issue's own chat, where the crew works it, and the turn that started there.
    ADD COLUMN session_id UUID REFERENCES sessions(id) ON DELETE SET NULL,
    ADD COLUMN turn_job_id UUID,
    ADD CONSTRAINT project_issues_dates_check CHECK (start_on IS NULL OR due_on IS NULL OR start_on <= due_on);

CREATE INDEX project_issues_crew_idx ON project_issues (crew_state) WHERE crew_state IN ('queued', 'working');
CREATE INDEX project_issues_session_idx ON project_issues (session_id) WHERE session_id IS NOT NULL;

-- Every project's two fixed columns, and its open issues in Backlog in the order they were filed.
INSERT INTO project_issue_columns (project_id, name, position, fixed)
SELECT id, 'Backlog', 0, 'backlog' FROM projects
UNION ALL
SELECT id, 'Ready to dev', 1, 'ready' FROM projects;

UPDATE project_issues i SET column_id = c.id, position = i.number
FROM project_issue_columns c
WHERE c.project_id = i.project_id AND c.fixed = 'backlog';

-- The project a chat works on: the project's own chat, or an issue's chat inside it.
CREATE FUNCTION chat_project(chat UUID) RETURNS UUID LANGUAGE sql STABLE AS $$
    SELECT COALESCE(
        (SELECT id FROM projects WHERE session_id = chat ORDER BY created_at DESC LIMIT 1),
        (SELECT project_id FROM project_issues WHERE session_id = chat LIMIT 1)
    )
$$;
