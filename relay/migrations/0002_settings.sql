-- Relay's settings, set from Nomi's admin: one row, the whole set sealed with
-- RELAY_ENCRYPTION_KEY (it holds database passwords and PostgREST's secret).
CREATE TABLE relay.settings (
    id BOOLEAN PRIMARY KEY DEFAULT true CHECK (id),
    sealed BYTEA NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
