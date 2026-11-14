-- How a provider key fared on the last call made with it: 'ok', 'balance_empty' (the provider
-- wants payment) or 'rejected' (the key was refused). checked_at is NULL until a call is made.
ALTER TABLE llm_provider_connections
    ADD COLUMN key_status TEXT NOT NULL DEFAULT 'ok' CHECK (key_status IN ('ok', 'balance_empty', 'rejected')),
    ADD COLUMN key_checked_at TIMESTAMPTZ;
