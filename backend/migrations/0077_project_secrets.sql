-- The secrets a project's admins stored for their published app (API keys and the like). Only
-- the names live here, to bind them into the app's environment when it's published: the values
-- are kept by Shipyard and never come back to Nomi or its models.
CREATE TABLE project_secrets (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, name)
);
