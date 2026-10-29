-- Each project's buckets (with who may read and write them) and its realtime tables, per
-- environment, as nomi/backend.json last said. Nothing secret, so not sealed.
CREATE TABLE relay.storage_configs (
    project_id UUID NOT NULL REFERENCES relay.projects(id) ON DELETE CASCADE,
    env TEXT NOT NULL CHECK (env IN ('dev', 'production')),
    config JSONB NOT NULL,
    PRIMARY KEY (project_id, env)
);

CREATE TABLE relay.realtime_tables (
    project_id UUID NOT NULL REFERENCES relay.projects(id) ON DELETE CASCADE,
    env TEXT NOT NULL CHECK (env IN ('dev', 'production')),
    tables TEXT[] NOT NULL,
    PRIMARY KEY (project_id, env)
);
