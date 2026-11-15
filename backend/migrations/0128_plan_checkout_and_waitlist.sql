-- Moving to a plan: what it costs in rupiah (NULL: not sold yet, so people join its waitlist),
-- each checkout started with the payment gateway and what came of it, and who asked to be told
-- when a plan opens.
ALTER TABLE plans ADD COLUMN price_idr BIGINT CHECK (price_idr IS NULL OR price_idr > 0);

CREATE TABLE plan_payments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id         UUID NOT NULL REFERENCES plans(id),
    -- What the gateway knows it by: ours, and the gateway's own invoice id.
    external_id     TEXT NOT NULL UNIQUE,
    invoice_id      TEXT,
    invoice_url     TEXT,
    amount_idr      BIGINT NOT NULL CHECK (amount_idr > 0),
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'paid', 'expired', 'failed')),
    -- How it was paid ("BANK_TRANSFER · BCA", "EWALLET · OVO"), once paid.
    payment_method  TEXT,
    paid_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX plan_payments_user ON plan_payments (user_id, created_at DESC);

CREATE TABLE plan_waitlist (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id     UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, plan_id)
);
