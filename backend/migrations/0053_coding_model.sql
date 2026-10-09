-- Which model Koda (the coding agent) builds with. An admin marks one of Nomi's models as the
-- coding model; each person can pick another of Nomi's models for their own projects, or have
-- Koda use the same model as their chats.

ALTER TABLE admin_llm_models ADD COLUMN is_coding_model BOOLEAN NOT NULL DEFAULT false;
CREATE UNIQUE INDEX admin_llm_models_one_coding_model ON admin_llm_models (is_coding_model) WHERE is_coding_model;

-- Both unset: Nomi's coding model (or their chat model when there's none).
ALTER TABLE user_llm_selections ADD COLUMN coding_admin_model_id UUID REFERENCES admin_llm_models(id) ON DELETE SET NULL;
ALTER TABLE user_llm_selections ADD COLUMN coding_same_as_chat BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE user_llm_selections ADD CONSTRAINT one_coding_choice CHECK (NOT (coding_same_as_chat AND coding_admin_model_id IS NOT NULL));
