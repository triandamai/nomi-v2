-- admin_llm_models.provider and user_llm_selections.custom_provider have carried a stale
-- CHECK constraint since 0012_admin_llm_models.sql: it only ever listed
-- ('anthropic', 'openai', 'gemini', 'fake'). OpenRouter was added as a selectable provider
-- (nomi-llm::ProviderKind::OpenRouter, ALLOWED_PROVIDERS in routes/llm_models.rs) without ever
-- updating this constraint, so every attempt to save an OpenRouter admin model or personal BYOK
-- selection has always failed with a silent 500 at the DB layer. DeepSeek hit the exact same gap.
ALTER TABLE admin_llm_models DROP CONSTRAINT admin_llm_models_provider_check;
ALTER TABLE admin_llm_models ADD CONSTRAINT admin_llm_models_provider_check
    CHECK (provider IN ('anthropic', 'openai', 'openrouter', 'gemini', 'deepseek', 'fake'));

ALTER TABLE user_llm_selections DROP CONSTRAINT user_llm_selections_custom_provider_check;
ALTER TABLE user_llm_selections ADD CONSTRAINT user_llm_selections_custom_provider_check
    CHECK (custom_provider IN ('anthropic', 'openai', 'openrouter', 'gemini', 'deepseek', 'fake'));
