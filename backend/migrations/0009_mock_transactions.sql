CREATE TABLE mock_transactions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id),
    occurred_at  TIMESTAMPTZ NOT NULL,
    amount_cents BIGINT NOT NULL,
    category     TEXT NOT NULL,
    description  TEXT NOT NULL
);
