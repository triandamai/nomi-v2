-- What Nomi tells a person outside a chat: plan and quota changes, promos, notices. Shown in the
-- app's inbox and, when they allow it, emailed.
CREATE TABLE notification_broadcasts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title       TEXT NOT NULL,
    body        TEXT NOT NULL,
    link        TEXT,
    -- 'all', or a plan's id: who it went to.
    audience    TEXT NOT NULL DEFAULT 'all',
    emailed     BOOLEAN NOT NULL DEFAULT false,
    recipients  INT NOT NULL DEFAULT 0,
    sent_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE notifications (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind          TEXT NOT NULL CHECK (kind IN ('account', 'subscription', 'quota', 'promo')),
    title         TEXT NOT NULL,
    body          TEXT NOT NULL,
    -- Where tapping it goes, inside the app (e.g. /billing).
    link          TEXT,
    broadcast_id  UUID REFERENCES notification_broadcasts(id) ON DELETE SET NULL,
    read_at       TIMESTAMPTZ,
    emailed_at    TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX notifications_user_created ON notifications (user_id, created_at DESC);
CREATE INDEX notifications_user_unread ON notifications (user_id) WHERE read_at IS NULL;

-- Which emails a person wants. Account emails cover plan, quota and other changes to their account.
ALTER TABLE user_preferences ADD COLUMN email_account BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN email_promos BOOLEAN NOT NULL DEFAULT true;
