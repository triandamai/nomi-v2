-- A phone's browser, subscribed to Web Push (VAPID). One row per push-service endpoint: the same
-- browser subscribing again refreshes its row, and an endpoint another person subscribed with
-- before is theirs no longer. A push service that answers 404 or 410 has the row deleted.
CREATE TABLE web_push_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    endpoint TEXT NOT NULL UNIQUE,
    p256dh TEXT NOT NULL,
    auth TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX web_push_subscriptions_user_id_idx ON web_push_subscriptions (user_id);
