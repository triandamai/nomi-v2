-- Each project's sign-in settings per environment (methods, redirect URLs, session length, its
-- own SMTP server with password), sealed with RELAY_ENCRYPTION_KEY.
CREATE TABLE relay.auth_configs (
    project_id UUID NOT NULL REFERENCES relay.projects(id) ON DELETE CASCADE,
    env TEXT NOT NULL CHECK (env IN ('dev', 'production')),
    sealed BYTEA NOT NULL,
    PRIMARY KEY (project_id, env)
);
