-- Plans people subscribe to, managed in Admin → Plans. Each gives a monthly allowance of tokens on
-- Nomi's own models; the card's look, price and promo are what the upgrade sheet shows.
CREATE TABLE plans (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug               TEXT NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9-]+$'),
    name               TEXT NOT NULL,
    description        TEXT NOT NULL DEFAULT '',
    monthly_tokens     BIGINT NOT NULL CHECK (monthly_tokens > 0),
    -- Shown as written, e.g. "Free" or "Rp 49.000 / month".
    price_label        TEXT NOT NULL DEFAULT '',
    features           TEXT[] NOT NULL DEFAULT '{}',
    -- One of the app's gradient tones (glow, ember, sky, tide, bloom, citrus, dusk, slate).
    card_tone          TEXT NOT NULL DEFAULT 'glow',
    promo_label        TEXT,
    promo_price_label  TEXT,
    promo_ends_at      TIMESTAMPTZ,
    -- Where new people start (exactly one plan).
    is_default         BOOLEAN NOT NULL DEFAULT false,
    -- Listed for people to see.
    is_active          BOOLEAN NOT NULL DEFAULT true,
    sort_order         INT NOT NULL DEFAULT 0,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX plans_one_default ON plans (is_default) WHERE is_default;

INSERT INTO plans (slug, name, description, monthly_tokens, price_label, features, card_tone, is_default, sort_order) VALUES
    ('free', 'Free', 'Everything Nomi does, with a monthly allowance.', 1000000, 'Free',
     ARRAY['1,000,000 tokens a month', 'The whole crew', 'Bring your own key anytime'], 'glow', true, 0),
    ('pro', 'Pro', 'Ten times the room for people who lean on Nomi every day.', 10000000, 'Coming soon',
     ARRAY['10,000,000 tokens a month', 'The whole crew', 'Priority for new features'], 'dusk', false, 1);

-- Each person's plan, and an admin's override of its allowance (until a date, or for good).
-- Someone with no row is on the default plan.
CREATE TABLE user_subscriptions (
    user_id         UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    plan_id         UUID NOT NULL REFERENCES plans(id),
    quota_override  BIGINT CHECK (quota_override IS NULL OR quota_override >= 0),
    override_until  TIMESTAMPTZ,
    note            TEXT,
    updated_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX user_subscriptions_plan ON user_subscriptions (plan_id);

-- Every change an admin made to someone's plan or allowance.
CREATE TABLE subscription_changes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id         UUID NOT NULL REFERENCES plans(id),
    quota_override  BIGINT,
    override_until  TIMESTAMPTZ,
    note            TEXT,
    changed_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX subscription_changes_user ON subscription_changes (user_id, created_at DESC);

-- Quota alerts already sent, so each goes out once a month: 'warning' at 80%, 'used_up' at 100%,
-- 'own_key' when Nomi switched to the person's own key.
CREATE TABLE quota_alerts (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    month       TEXT NOT NULL,
    level       TEXT NOT NULL CHECK (level IN ('warning', 'used_up', 'own_key')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, month, level)
);

-- A person's own API key now stays saved when they switch back to one of Nomi's models, so Nomi
-- can fall back to it when their allowance runs out. The custom_* columns hold the saved key;
-- admin_model_id set means Nomi's model is the one in use.
ALTER TABLE user_llm_selections DROP CONSTRAINT exactly_one_source;
