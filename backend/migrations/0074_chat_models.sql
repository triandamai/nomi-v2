-- The model a chat runs on, when someone picked one for it in the chat (the model selector):
-- one of Nomi's models, or a model from the picker's own connected provider. Neither set means
-- the person's default (Models). Removing the model or the provider forgets the pick.
ALTER TABLE sessions
    ADD COLUMN model_admin_id      UUID REFERENCES admin_llm_models(id) ON DELETE SET NULL,
    ADD COLUMN model_connection_id UUID REFERENCES llm_provider_connections(id) ON DELETE SET NULL,
    ADD COLUMN model_id            TEXT,
    ADD COLUMN model_label         TEXT,
    -- Whose pick it is: a model from someone's own provider only runs their own turns.
    ADD COLUMN model_set_by        UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD CONSTRAINT sessions_one_model_source CHECK (model_admin_id IS NULL OR model_connection_id IS NULL);

-- Which models each person's selector shows (Manage models). Only changes from the defaults are
-- stored: Nomi's models are shown and models from their own providers aren't, until switched.
-- model_key is "admin:<id>" or "conn:<connection id>:<model id>".
CREATE TABLE user_model_visibility (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    model_key   TEXT NOT NULL,
    enabled     BOOLEAN NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, model_key)
);
