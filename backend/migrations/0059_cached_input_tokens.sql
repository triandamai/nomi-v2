-- Input a provider served from its prompt cache: counted in `input_tokens` too, but priced at
-- the model's cached-input price (often a tenth of the full price). An empty cached price means
-- cached input costs the same as any other input.
ALTER TABLE llm_usage ADD COLUMN cached_input_tokens INTEGER NOT NULL DEFAULT 0 CHECK (cached_input_tokens >= 0);
ALTER TABLE admin_llm_models ADD COLUMN cached_input_usd_per_mtok NUMERIC(12, 4) CHECK (cached_input_usd_per_mtok >= 0);
