-- How Nomi reaches Nomi Relay, set in the admin: Relay's address and its control key (encrypted
-- with SETTINGS_ENCRYPTION_KEY). One row; RELAY_CONTROL_URL / RELAY_CONTROL_KEY are the fallback.
CREATE TABLE relay_connection (
    id BOOLEAN PRIMARY KEY DEFAULT true CHECK (id),
    control_url TEXT NOT NULL,
    control_key_encrypted BYTEA NOT NULL,
    updated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
