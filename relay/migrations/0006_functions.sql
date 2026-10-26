-- Each deploy of a project environment's nomi/functions/: its files, the settings backend.json
-- gave each function, and whether it's the build being served. Numbered per environment; a build
-- that failed its check keeps its error and is never served.
CREATE TABLE relay.function_builds (
    project_id UUID NOT NULL REFERENCES relay.projects(id) ON DELETE CASCADE,
    env TEXT NOT NULL CHECK (env IN ('dev', 'production')),
    version INT NOT NULL,
    files JSONB NOT NULL,
    functions JSONB NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('checking', 'current', 'replaced', 'failed')),
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, env, version)
);
CREATE UNIQUE INDEX function_builds_one_current ON relay.function_builds (project_id, env) WHERE status = 'current';

-- Scheduled runs claimed, one row per function and minute, so a run happens once however many
-- Relay instances are up.
CREATE TABLE relay.function_runs (
    project_id UUID NOT NULL REFERENCES relay.projects(id) ON DELETE CASCADE,
    env TEXT NOT NULL,
    name TEXT NOT NULL,
    minute TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (project_id, env, name, minute)
);
