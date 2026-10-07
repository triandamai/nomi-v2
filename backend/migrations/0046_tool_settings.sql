-- Admin → Tools: every tool's switch and settings. A tool with no row is on with its defaults.
-- `secrets_encrypted` holds API keys (an encrypted JSON object), never sent back to the browser.
CREATE TABLE tool_settings (
    name              TEXT PRIMARY KEY,
    enabled           BOOLEAN NOT NULL DEFAULT true,
    config            JSONB NOT NULL DEFAULT '{}',
    secrets_encrypted BYTEA,
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
