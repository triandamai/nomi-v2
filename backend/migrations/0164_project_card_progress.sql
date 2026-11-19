-- A publish the person stopped: it never finished, and a late webhook can't revive it (the
-- webhook and the overview only update publishes that are still 'running').
ALTER TABLE project_publishes DROP CONSTRAINT project_publishes_status_check;
ALTER TABLE project_publishes ADD CONSTRAINT project_publishes_status_check
    CHECK (status IN ('running', 'succeeded', 'failed', 'cancelled'));

-- How many builds (publishes) a plan includes each calendar month. NULL is unlimited, like the
-- plan's other limits, so nothing is refused or counted down until an admin sets one.
ALTER TABLE plans ADD COLUMN max_builds_month INT CHECK (max_builds_month IS NULL OR max_builds_month >= 0);

-- A move into a workspace the person doesn't own waits here until that workspace's owner
-- approves or declines it. Moving within the person's own workspaces stays immediate.
CREATE TABLE project_transfer_requests (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id    UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    from_org_id   UUID NOT NULL REFERENCES organizations(id),
    to_org_id     UUID NOT NULL REFERENCES organizations(id),
    requested_by  UUID NOT NULL REFERENCES users(id),
    status        TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'declined', 'cancelled')),
    decided_by    UUID REFERENCES users(id),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    decided_at    TIMESTAMPTZ
);
-- One open request per project.
CREATE UNIQUE INDEX project_transfer_requests_one_pending ON project_transfer_requests (project_id) WHERE status = 'pending';
CREATE INDEX project_transfer_requests_target ON project_transfer_requests (to_org_id, status);
