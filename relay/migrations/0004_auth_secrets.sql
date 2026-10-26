-- A project's sign-in secrets by name (NOMI_SMTP_PASSWORD, NOMI_OAUTH_<PROVIDER>_SECRET,
-- NOMI_OAUTH_APPLE_KEY), sent by Nomi when an admin sets them; sealed with RELAY_ENCRYPTION_KEY.
CREATE TABLE relay.auth_secrets (
    project_id UUID NOT NULL REFERENCES relay.projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    sealed BYTEA NOT NULL,
    PRIMARY KEY (project_id, name)
);
