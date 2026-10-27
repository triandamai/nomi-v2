-- Nomi's configuration as set in Admin -> Configuration: one row per field. Secrets are encrypted
-- with SETTINGS_ENCRYPTION_KEY; the matching environment variable is the fallback for anything not
-- saved here.
CREATE TABLE app_config (
    section TEXT NOT NULL,
    field TEXT NOT NULL,
    value TEXT,
    value_encrypted BYTEA,
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (section, field),
    CHECK ((value IS NULL) <> (value_encrypted IS NULL))
);
