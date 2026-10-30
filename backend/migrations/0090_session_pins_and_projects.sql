-- A chat the person pinned to the top of their list, and since when (newest pin first).
ALTER TABLE sessions ADD COLUMN pinned_at TIMESTAMPTZ;

-- The project a chat was moved into. A project's own chat is `projects.session_id`; this files any
-- other chat under a project. It goes back to no project when the project is deleted.
ALTER TABLE sessions ADD COLUMN in_project_id UUID REFERENCES projects(id) ON DELETE SET NULL;
CREATE INDEX sessions_in_project_id_idx ON sessions (in_project_id) WHERE in_project_id IS NOT NULL;
