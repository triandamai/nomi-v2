-- What each model costs, so spend can be shown per person (USD per million tokens). Empty means
-- the price isn't set yet: usage still counts, spend reads as zero.
ALTER TABLE admin_llm_models
    ADD COLUMN input_usd_per_mtok  NUMERIC(12, 4) CHECK (input_usd_per_mtok >= 0),
    ADD COLUMN output_usd_per_mtok NUMERIC(12, 4) CHECK (output_usd_per_mtok >= 0);

-- One row per call to a language model, for the person it was made for. `source` says whose key
-- paid: 'nomi' (an admin model, priced above) or 'own_key' (the person's own API key, which
-- Nomi never bills). The cost is fixed when the call is made, so later price changes don't
-- rewrite history.
CREATE TABLE llm_usage (
    id             BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    admin_model_id UUID REFERENCES admin_llm_models(id) ON DELETE SET NULL,
    source         TEXT NOT NULL CHECK (source IN ('nomi', 'own_key')),
    model_label    TEXT NOT NULL,
    provider       TEXT NOT NULL,
    model_id       TEXT NOT NULL,
    input_tokens   INTEGER NOT NULL CHECK (input_tokens >= 0),
    output_tokens  INTEGER NOT NULL CHECK (output_tokens >= 0),
    cost_usd       NUMERIC(16, 8) NOT NULL DEFAULT 0,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX llm_usage_user_time ON llm_usage (user_id, created_at);
