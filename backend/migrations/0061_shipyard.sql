-- Publishing projects on Shipyard: each project is a Shipyard app, each publish keeps its source
-- so it can be rolled back to, and Shipyard's webhook events are kept per project.

-- The Shipyard app a project became on its first publish.
ALTER TABLE projects ADD COLUMN shipyard_app TEXT UNIQUE;
-- Who can open a project: everyone active in its organization, or only the people added to it.
ALTER TABLE projects ADD COLUMN access TEXT NOT NULL DEFAULT 'org' CHECK (access IN ('org', 'members'));

-- When the organization was created on Shipyard.
ALTER TABLE organizations ADD COLUMN shipyard_synced_at TIMESTAMPTZ;

-- People added to a project. Admins publish, roll back, take it offline and manage members;
-- members build it. The person who started a project is always its admin.
CREATE TABLE project_members (
    project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role        TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('admin', 'member')),
    added_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, user_id)
);
CREATE INDEX project_members_user_idx ON project_members (user_id);

-- A project is open to its starter, and to active members of its organization: all of them,
-- or with `access = 'members'` only those added to it.
CREATE OR REPLACE FUNCTION can_open_project(project UUID, person UUID) RETURNS boolean LANGUAGE sql STABLE AS $$
    SELECT EXISTS (
        SELECT 1 FROM projects p
        WHERE p.id = project
          AND (p.user_id = person OR (
              EXISTS (SELECT 1 FROM memberships m WHERE m.org_id = p.org_id AND m.user_id = person AND m.status = 'active')
              AND (p.access = 'org' OR EXISTS (SELECT 1 FROM project_members pm WHERE pm.project_id = p.id AND pm.user_id = person))
          ))
    )
$$;

-- Every publish of a project to an environment, newest last.
CREATE TABLE project_publishes (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id          UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    environment         TEXT NOT NULL DEFAULT 'production',
    shipyard_publish_id TEXT UNIQUE,
    status              TEXT NOT NULL DEFAULT 'running' CHECK (status IN ('running', 'succeeded', 'failed')),
    url                 TEXT,
    error               TEXT,
    -- The zip that was sent, in project storage, so it can be sent again.
    source_key          TEXT,
    manifest            JSONB NOT NULL,
    files_version       BIGINT NOT NULL DEFAULT 0,
    -- The publish this one rolled back to.
    rollback_of         UUID REFERENCES project_publishes(id) ON DELETE SET NULL,
    created_by          UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at         TIMESTAMPTZ
);
CREATE INDEX project_publishes_project_idx ON project_publishes (project_id, created_at DESC);

-- Shipyard's webhook events, deduplicated on its event id.
CREATE TABLE shipyard_events (
    id           TEXT PRIMARY KEY,
    type         TEXT NOT NULL,
    project_id   UUID REFERENCES projects(id) ON DELETE CASCADE,
    environment  TEXT,
    service      TEXT,
    data         JSONB NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL,
    received_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX shipyard_events_project_idx ON shipyard_events (project_id, created_at DESC);

-- The webhook endpoint Nomi registered on Shipyard for itself, and its signing secret.
CREATE TABLE shipyard_webhook (
    singleton         BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    endpoint_id       TEXT NOT NULL,
    url               TEXT NOT NULL,
    secret_encrypted  BYTEA NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Deploy problems reach the project's admins.
ALTER TABLE notifications DROP CONSTRAINT IF EXISTS notifications_kind_check;
ALTER TABLE notifications ADD CONSTRAINT notifications_kind_check
    CHECK (kind IN ('account', 'subscription', 'quota', 'promo', 'deploy'));
