-- Providers connected once (Providers, for the whole instance or for one person) whose models are
-- then picked, instead of a key on every model. provider_id names an entry in nomi-llm's catalog
-- (checked in code, so the catalog can grow without a migration); kind is the client Nomi uses.
CREATE TABLE llm_provider_connections (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- NULL: the instance's own (admin) connection, shared by everyone.
    user_id             UUID REFERENCES users(id) ON DELETE CASCADE,
    provider_id         TEXT NOT NULL,
    label               TEXT NOT NULL,
    kind                TEXT NOT NULL CHECK (kind IN ('anthropic', 'openai', 'openrouter', 'gemini', 'deepseek', 'openai_compatible', 'fake')),
    auth_type           TEXT NOT NULL CHECK (auth_type IN ('api_key', 'custom_endpoint')),
    api_key_encrypted   BYTEA,
    base_url            TEXT,
    -- A custom endpoint's extra headers, as encrypted JSON (values can be secrets).
    headers_encrypted   BYTEA,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by          UUID REFERENCES users(id) ON DELETE SET NULL
);
-- One connection per provider in each scope; custom endpoints repeat, each under its own name.
CREATE UNIQUE INDEX llm_provider_connections_provider_idx
    ON llm_provider_connections (COALESCE(user_id, '00000000-0000-0000-0000-000000000000'::uuid), provider_id)
    WHERE provider_id <> 'custom';
CREATE UNIQUE INDEX llm_provider_connections_custom_label_idx
    ON llm_provider_connections (COALESCE(user_id, '00000000-0000-0000-0000-000000000000'::uuid), lower(label))
    WHERE provider_id = 'custom';
CREATE INDEX llm_provider_connections_user_idx ON llm_provider_connections (user_id);

-- A model can come from a connection. A model that still has its own key keeps using it.
ALTER TABLE admin_llm_models
    ADD COLUMN connection_id UUID REFERENCES llm_provider_connections(id) ON DELETE RESTRICT,
    ALTER COLUMN api_key_encrypted DROP NOT NULL;
CREATE INDEX admin_llm_models_connection_idx ON admin_llm_models (connection_id);

ALTER TABLE user_llm_selections
    ADD COLUMN custom_connection_id UUID REFERENCES llm_provider_connections(id) ON DELETE SET NULL;

ALTER TABLE admin_llm_models DROP CONSTRAINT IF EXISTS admin_llm_models_provider_check;
ALTER TABLE admin_llm_models ADD CONSTRAINT admin_llm_models_provider_check
    CHECK (provider IN ('anthropic', 'openai', 'openrouter', 'gemini', 'deepseek', 'openai_compatible', 'fake'));
ALTER TABLE user_llm_selections DROP CONSTRAINT IF EXISTS user_llm_selections_custom_provider_check;
ALTER TABLE user_llm_selections ADD CONSTRAINT user_llm_selections_custom_provider_check
    CHECK (custom_provider IN ('anthropic', 'openai', 'openrouter', 'gemini', 'deepseek', 'openai_compatible', 'fake'));

-- Turns keys saved the old way into connections: one instance connection per provider the admin
-- models use (key and base URL from its most recently updated model) and one personal connection
-- per saved own key. Models keep their own keys, so nothing changes for anyone. Safe to run again
-- (only rows not linked yet), e.g. after an older version wrote new rows during a rollback.
CREATE FUNCTION backfill_provider_connections() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO llm_provider_connections (user_id, provider_id, label, kind, auth_type, api_key_encrypted, base_url, updated_by)
    SELECT DISTINCT ON (m.provider)
        NULL,
        CASE m.provider WHEN 'gemini' THEN 'google' ELSE m.provider END,
        CASE m.provider WHEN 'anthropic' THEN 'Anthropic' WHEN 'openai' THEN 'OpenAI' WHEN 'gemini' THEN 'Google'
            WHEN 'openrouter' THEN 'OpenRouter' WHEN 'deepseek' THEN 'DeepSeek' END,
        m.provider, 'api_key', m.api_key_encrypted, m.base_url, m.updated_by
    FROM admin_llm_models m
    WHERE m.connection_id IS NULL AND m.api_key_encrypted IS NOT NULL
      AND m.provider IN ('anthropic', 'openai', 'gemini', 'openrouter', 'deepseek')
    ORDER BY m.provider, m.updated_at DESC
    ON CONFLICT DO NOTHING;

    UPDATE admin_llm_models m SET connection_id = c.id
    FROM llm_provider_connections c
    WHERE m.connection_id IS NULL AND c.user_id IS NULL
      AND c.provider_id = CASE m.provider WHEN 'gemini' THEN 'google' ELSE m.provider END;

    INSERT INTO llm_provider_connections (user_id, provider_id, label, kind, auth_type, api_key_encrypted, base_url, updated_by)
    SELECT s.user_id,
        CASE s.custom_provider WHEN 'gemini' THEN 'google' ELSE s.custom_provider END,
        CASE s.custom_provider WHEN 'anthropic' THEN 'Anthropic' WHEN 'openai' THEN 'OpenAI' WHEN 'gemini' THEN 'Google'
            WHEN 'openrouter' THEN 'OpenRouter' WHEN 'deepseek' THEN 'DeepSeek' END,
        s.custom_provider, 'api_key', s.custom_api_key_encrypted, s.custom_base_url, s.user_id
    FROM user_llm_selections s
    WHERE s.custom_connection_id IS NULL AND s.custom_api_key_encrypted IS NOT NULL
      AND s.custom_provider IN ('anthropic', 'openai', 'gemini', 'openrouter', 'deepseek')
    ON CONFLICT DO NOTHING;

    UPDATE user_llm_selections s SET custom_connection_id = c.id
    FROM llm_provider_connections c
    WHERE s.custom_connection_id IS NULL AND s.custom_api_key_encrypted IS NOT NULL AND c.user_id = s.user_id
      AND c.provider_id = CASE s.custom_provider WHEN 'gemini' THEN 'google' ELSE s.custom_provider END;
END;
$$;

SELECT backfill_provider_connections();
