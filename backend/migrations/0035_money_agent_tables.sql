-- The Money agent's own tables (agent-owned storage: see docs/agents/storage.md). Only
-- nomi-agent-money and the Money page's routes write them.

-- Transactions move into a Money-owned table. `mock_transactions` stays as a view over it, so
-- anything still reading or inserting through the old name keeps working.
ALTER TABLE mock_transactions RENAME TO money_transactions;
ALTER TABLE money_transactions
    ADD COLUMN source TEXT NOT NULL DEFAULT 'import' CHECK (source IN ('import', 'manual', 'agent')),
    ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT now();
CREATE INDEX money_transactions_user_time_idx ON money_transactions (user_id, occurred_at DESC);
CREATE VIEW mock_transactions AS
    SELECT id, user_id, occurred_at, amount_cents, category, description FROM money_transactions;

-- A monthly spending limit per category.
CREATE TABLE money_budgets (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id              UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category             TEXT NOT NULL,
    monthly_limit_cents  BIGINT NOT NULL CHECK (monthly_limit_cents > 0),
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, category)
);
