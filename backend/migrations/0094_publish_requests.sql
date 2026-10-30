-- Someone who can work on a project but not publish it asks its admins to publish; an admin
-- approves (which publishes, `publish_id`) or declines, or the person who asked takes it back.
-- One open request per person per project at a time.
CREATE TABLE publish_requests (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id   UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    environment  TEXT NOT NULL DEFAULT 'production',
    note         TEXT,
    status       TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'declined', 'cancelled')),
    decided_by   UUID REFERENCES users(id) ON DELETE SET NULL,
    decided_at   TIMESTAMPTZ,
    publish_id   UUID REFERENCES project_publishes(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX publish_requests_project_idx ON publish_requests (project_id, created_at DESC);
CREATE UNIQUE INDEX publish_requests_one_open_idx ON publish_requests (project_id, requested_by) WHERE status = 'pending';
