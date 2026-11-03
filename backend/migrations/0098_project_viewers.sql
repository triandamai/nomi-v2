-- A third place on a project: someone who can open it and watch (the chat, the preview, the
-- code) but changes nothing: no messages to the crew, no file edits, no publish requests.
ALTER TABLE project_members DROP CONSTRAINT IF EXISTS project_members_role_check;
ALTER TABLE project_members ADD CONSTRAINT project_members_role_check CHECK (role IN ('admin', 'member', 'viewer'));
