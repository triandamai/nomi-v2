-- Every app a person signs in to is a device: phones, computers, and each browser. The app makes
-- its id once and keeps it; it is also the MQTT client id of the app's realtime connection.
ALTER TABLE devices DROP CONSTRAINT devices_platform_check;
ALTER TABLE devices ADD CONSTRAINT devices_platform_check
    CHECK (platform IN ('android', 'ios', 'macos', 'windows', 'linux', 'web'));
-- What the person sees in their device list: "Pixel 9", "Chrome on macOS".
ALTER TABLE devices ADD COLUMN name TEXT;
ALTER TABLE devices ADD COLUMN brand TEXT;
ALTER TABLE devices ADD COLUMN model TEXT;
ALTER TABLE devices ADD COLUMN os_name TEXT;
ALTER TABLE devices ADD COLUMN os_version TEXT;
-- online: connected with the app in front; away: connected in the background; offline: neither.
ALTER TABLE devices ADD COLUMN presence TEXT NOT NULL DEFAULT 'offline'
    CHECK (presence IN ('online', 'away', 'offline'));

-- The device a sign-in was made on, so signing that device out ends it. Not a foreign key: an app
-- signs in before it registers its device.
ALTER TABLE refresh_tokens ADD COLUMN device_id UUID;
CREATE INDEX refresh_tokens_device_id_idx ON refresh_tokens (device_id) WHERE device_id IS NOT NULL;

-- A device's live connections (a browser has one per tab); the device is online while one is in
-- front, away while all are in the background, offline when none is left.
CREATE TABLE device_connections (
    client_id  TEXT PRIMARY KEY,
    device_id  UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    -- Which connection under this client id: a reconnect replaces the old one, whose `offline`
    -- may arrive after the new one's `online`.
    connection UUID NOT NULL,
    state      TEXT NOT NULL CHECK (state IN ('online', 'away')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX device_connections_device_id_idx ON device_connections (device_id);
