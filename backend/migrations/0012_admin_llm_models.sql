CREATE TABLE admin_llm_models (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    label              TEXT NOT NULL,
    provider           TEXT NOT NULL CHECK (provider IN ('anthropic', 'openai', 'gemini', 'fake')),
    model_id           TEXT NOT NULL,
    api_key_encrypted  BYTEA NOT NULL,
    base_url           TEXT,
    is_default         BOOLEAN NOT NULL DEFAULT false,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by         UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE user_llm_selections (
    user_id                    UUID PRIMARY KEY REFERENCES users(id),
    admin_model_id             UUID REFERENCES admin_llm_models(id) ON DELETE SET NULL,
    custom_label               TEXT,
    custom_provider            TEXT CHECK (custom_provider IN ('anthropic', 'openai', 'gemini', 'fake')),
    custom_model_id            TEXT,
    custom_api_key_encrypted   BYTEA,
    custom_base_url            TEXT,
    updated_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT exactly_one_source CHECK (
        (admin_model_id IS NOT NULL AND custom_provider IS NULL) OR
        (admin_model_id IS NULL AND custom_provider IS NOT NULL) OR
        (admin_model_id IS NULL AND custom_provider IS NULL)
    )
);

-- Seed from the existing single-row LLM config, if one was ever configured, so upgrading
-- doesn't strand every user without a working model. Fresh installs get no seed row; the
-- env-var fallback in resolve_llm_model_config covers that until an admin adds one.
INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, base_url, is_default, updated_by)
SELECT 'Default', provider, model_id, api_key_encrypted, base_url, true, updated_by
FROM provider_settings
WHERE setting_type = 'llm';
