-- The phones and computers a person's native apps run on. Phones carry a push token (FCM on
-- Android, APNs on iOS) so inbox items and chat replies reach them when no socket can stay open;
-- desktops register without one and are kept only for "last seen".
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform TEXT NOT NULL CHECK (platform IN ('android', 'ios', 'macos', 'windows', 'linux')),
    push_token TEXT,
    -- iOS only: ActivityKit push-to-start token, so the server can start a Live Activity.
    live_activity_start_token TEXT,
    app_version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, push_token)
);
CREATE INDEX devices_user_id_idx ON devices (user_id);
