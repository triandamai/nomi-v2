-- Money: income as well as spending, several items in one transaction (a receipt's lines), and
-- each person's own month (from payday on the 25th, say, instead of the 1st).
ALTER TABLE money_transactions
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'expense' CHECK (kind IN ('expense', 'income'));

-- The lines of a transaction. Its amount_cents stays the total (what the receipt says, tax and
-- discounts included); items show what it was made of.
CREATE TABLE money_transaction_items (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id     UUID NOT NULL REFERENCES money_transactions(id) ON DELETE CASCADE,
    user_id            UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    position           INT NOT NULL DEFAULT 0,
    name               TEXT NOT NULL,
    quantity           NUMERIC(12, 3) NOT NULL DEFAULT 1 CHECK (quantity > 0),
    unit_amount_cents  BIGINT,
    amount_cents       BIGINT NOT NULL CHECK (amount_cents >= 0),
    -- Set when an item belongs to another category than its transaction.
    category           TEXT
);
CREATE INDEX money_transaction_items_transaction ON money_transaction_items (transaction_id, position);

-- The day each money month starts (1–28). 1 is the calendar month.
ALTER TABLE user_preferences
    ADD COLUMN money_period_start_day SMALLINT NOT NULL DEFAULT 1 CHECK (money_period_start_day BETWEEN 1 AND 28);
