CREATE TABLE provider_settings (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    setting_type       TEXT NOT NULL UNIQUE CHECK (setting_type IN ('llm', 'embedding')),
    provider           TEXT NOT NULL,
    model_id           TEXT NOT NULL,
    api_key_encrypted  BYTEA NOT NULL,
    base_url           TEXT,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by         UUID NOT NULL REFERENCES users(id)
);
