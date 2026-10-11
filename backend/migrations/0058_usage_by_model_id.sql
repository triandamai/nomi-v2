-- Usage is told apart by the admin model's id, not its name: renaming a model keeps its
-- history together, and a deleted model's rows keep the id they were made with (the label
-- recorded with each call names it then).
ALTER TABLE llm_usage DROP CONSTRAINT IF EXISTS llm_usage_admin_model_id_fkey;
CREATE INDEX llm_usage_time ON llm_usage (created_at);
CREATE INDEX llm_usage_model ON llm_usage (admin_model_id, created_at);

-- One row per text Nomi turns into an embedding (saving or looking up memories). `tokens` is
-- what the provider counted; `estimated` marks providers that don't report it (Gemini), where
-- it's worked out from the text's length.
CREATE TABLE embedding_usage (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    provider   TEXT NOT NULL,
    model_id   TEXT NOT NULL,
    tokens     INTEGER NOT NULL CHECK (tokens >= 0),
    estimated  BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX embedding_usage_time ON embedding_usage (created_at);
