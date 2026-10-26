-- Who has a project open right now: each open tab sends a heartbeat, and a person counts as
-- here until their last heartbeat is older than the expiry (see routes::project_presence). Rows
-- are overwritten, never accumulated; nothing about presence goes into messages.
CREATE TABLE project_presence (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_seen  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, user_id)
);
