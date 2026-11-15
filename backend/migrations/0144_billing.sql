-- Paid plans: the terms (months paid up front) with their discounts, coupons, referral codes, and
-- the payments (Xendit invoices) that activate a plan. A plan's monthly price is plans.price_idr
-- (0128; NULL: not sold yet, so people join its waitlist).
ALTER TABLE plans
    -- A cheaper monthly price in rupiah while promo_ends_at is in the future (or unset: no end).
    ADD COLUMN promo_price_idr BIGINT CHECK (promo_price_idr IS NULL OR promo_price_idr >= 0);

CREATE TABLE billing_terms (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    months           INT  NOT NULL UNIQUE CHECK (months > 0),
    discount_percent INT  NOT NULL DEFAULT 0 CHECK (discount_percent BETWEEN 0 AND 100),
    label            TEXT NOT NULL DEFAULT '',
    is_active        BOOLEAN NOT NULL DEFAULT true,
    sort_order       INT  NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
INSERT INTO billing_terms (months, discount_percent, label, sort_order) VALUES
    (1, 0, '1 month', 0),
    (6, 10, '6 months', 1),
    (12, 20, '12 months', 2);

ALTER TABLE user_subscriptions
    -- When the paid period ends; NULL for a plan an admin assigned (no end).
    ADD COLUMN paid_until   TIMESTAMPTZ,
    ADD COLUMN term_months  INT,
    ADD COLUMN auto_renew   BOOLEAN NOT NULL DEFAULT true,
    -- A referral reward waiting for the next paid period.
    ADD COLUMN bonus_days   INT NOT NULL DEFAULT 0 CHECK (bonus_days >= 0);

CREATE TABLE coupons (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Stored upper-case.
    code              TEXT NOT NULL UNIQUE CHECK (code = upper(code) AND length(code) > 0),
    description       TEXT,
    discount_kind     TEXT NOT NULL CHECK (discount_kind IN ('percent', 'amount')),
    discount_value    BIGINT NOT NULL CHECK (discount_value > 0 AND (discount_kind <> 'percent' OR discount_value <= 100)),
    max_redemptions   INT CHECK (max_redemptions IS NULL OR max_redemptions > 0),
    per_user_limit    INT NOT NULL DEFAULT 1 CHECK (per_user_limit > 0),
    starts_at         TIMESTAMPTZ,
    ends_at           TIMESTAMPTZ,
    -- Empty: any plan.
    plan_ids          UUID[] NOT NULL DEFAULT '{}',
    min_months        INT NOT NULL DEFAULT 1 CHECK (min_months > 0),
    first_payment_only BOOLEAN NOT NULL DEFAULT false,
    is_active         BOOLEAN NOT NULL DEFAULT true,
    created_by        UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE payments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id             UUID NOT NULL REFERENCES plans(id),
    term_months         INT  NOT NULL CHECK (term_months > 0),
    kind                TEXT NOT NULL CHECK (kind IN ('new', 'renewal')),
    base_amount         BIGINT NOT NULL,
    term_discount       BIGINT NOT NULL DEFAULT 0,
    promo_discount      BIGINT NOT NULL DEFAULT 0,
    coupon_discount     BIGINT NOT NULL DEFAULT 0,
    referral_discount   BIGINT NOT NULL DEFAULT 0,
    amount              BIGINT NOT NULL CHECK (amount >= 0),
    currency            TEXT NOT NULL,
    coupon_id           UUID REFERENCES coupons(id) ON DELETE SET NULL,
    referral_code       TEXT,
    status              TEXT NOT NULL CHECK (status IN ('pending', 'paid', 'expired', 'failed', 'cancelled')),
    provider            TEXT NOT NULL DEFAULT 'xendit',
    provider_invoice_id TEXT,
    invoice_url         TEXT,
    expires_at          TIMESTAMPTZ,
    paid_at             TIMESTAMPTZ,
    period_start        TIMESTAMPTZ,
    period_end          TIMESTAMPTZ,
    payment_method      TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX payments_user ON payments (user_id, created_at DESC);
CREATE INDEX payments_status ON payments (status, created_at DESC);
CREATE UNIQUE INDEX payments_provider_invoice ON payments (provider_invoice_id) WHERE provider_invoice_id IS NOT NULL;

-- A coupon is counted from the moment a payment is created with it, and released when that
-- payment expires, fails or is cancelled, so a cap counts pending and paid payments.
CREATE TABLE coupon_redemptions (
    coupon_id   UUID NOT NULL REFERENCES coupons(id),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    payment_id  UUID NOT NULL UNIQUE REFERENCES payments(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX coupon_redemptions_coupon ON coupon_redemptions (coupon_id, user_id);

CREATE TABLE referral_codes (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    code    TEXT NOT NULL UNIQUE
);

CREATE TABLE referrals (
    referee_user_id  UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    referrer_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    payment_id       UUID REFERENCES payments(id) ON DELETE SET NULL,
    rewarded_at      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX referrals_referrer ON referrals (referrer_user_id);

-- Billing reminders already sent, so each goes out once per paid period: 'reminder' (the period
-- ends soon and it won't renew by itself) and 'ended' (it ran out and the plan fell back).
CREATE TABLE billing_notices (
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind       TEXT NOT NULL CHECK (kind IN ('reminder', 'ended')),
    paid_until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, kind, paid_until)
);
