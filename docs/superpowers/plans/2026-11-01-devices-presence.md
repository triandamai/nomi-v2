# Devices and presence

Every app (Android, iOS, desktop, each browser) has a device id it keeps for good. The id is the
MQTT client id of the app's connection through the backend relays to EMQX. The backend saves each
device (brand, model, OS, app version) and its presence (online, away, offline). The web
account page lists the devices and can sign one out.

Realtime stays as it is: the backend publishes on EMQX, and apps reach it through the backend
WebSocket relays. Presence uses MQTT itself: each presence connection publishes its state, and its
MQTT last will says `offline` when it drops.

## Wire

- Every request carries `X-Device-Id: <uuid>`. A relay socket's MQTT client id is:
  - the device id, for the native realtime socket (`/api/realtime/ws`);
  - `<deviceId>-<tabId>`, for the web activity socket (`/api/me/activity/ws`, `X-Tab-Id`); browser
    tabs share a device id, and EMQX allows one connection per client id;
  - `<deviceId>-<purpose>-<random>`, for the chat, project and admin relays;
  - the old random id, when the request has no valid device id.
- Presence topic `presence/{user_id}/{client_id}`: payload `{"device_id","state"}`, where `state`
  is `online`, `away` or `offline`. The relay publishes `online` on connect, then what the app
  sends (`{"op":"presence","state":"away"|"online"}`), then `offline` on a clean close. Its last
  will is `offline`, for a crash or a dropped connection.
- User topic (`user/{id}/activity`):
  - `DevicePresence {device_id, presence, last_seen_at}`;
  - `DeviceSignedOut {device_id}`.

## Backend (`backend/`)

1. Migration `0096_devices_presence.sql`:
   - `devices` gains `name`, `brand`, `model`, `os_name`, `os_version`, and
     `presence` (`online`/`away`/`offline`, default `offline`);
   - the platform list adds `web`;
   - `refresh_tokens` gains `device_id UUID`;
   - new table `device_connections(client_id PK, device_id FK cascade, state, updated_at)`.
2. `nomi-auth`: `issue_refresh_token` and `refresh_access_token` take the device id; a refresh
   tags an older token with it. Login, refresh and Google sign-in read `X-Device-Id`.
3. `routes/devices.rs`:
   - `POST /api/devices` takes `device_id` plus the details and upserts by id; a device id or
     push token someone else had becomes this person's;
   - `GET /api/devices` lists the person's devices;
   - `DELETE /api/devices/:id` revokes that device's refresh tokens, deletes it, and publishes
     `DeviceSignedOut`.
4. `nomi-realtime`:
   - `device_client_id(...)`;
   - `PresenceState`;
   - the presence topic, payload and last will;
   - `UserEnvelope::DevicePresence` and `UserEnvelope::DeviceSignedOut`;
   - `ClientFrame::Presence`.
5. Relays (`routes/realtime.rs`, `sessions.rs` activity, chat and project relays,
   `admin_dashboard.rs`): client id from the device id. The two presence relays set the last
   will, publish their state, and read `presence` frames.
6. `presence.rs`, run by the server:
   - subscribes `presence/+/+`;
   - keeps `device_connections`;
   - sets `devices.presence` and `last_seen_at` with an `UPDATE … WHERE presence <> $new RETURNING`,
     so with several servers only one publishes `DevicePresence`.
7. Tests:
   - unit tests for client ids and payloads;
   - route tests for devices;
   - presence and relay tests (CI, against the broker).

## Rust SDK (`rust/crates/nomi-sdk`)

1. On first start, the device id is created in `data_dir/device_id`, then reused; `device_id()`
   exposes it.
2. `X-Device-Id` goes on every HTTP request (reqwest default header) and on the realtime handshake.
3. `set_device(DeviceInfo)`: kind, name, brand, model, OS name and version, app version. The SDK
   registers the device now when signed in, and after every sign-in.
4. `set_foreground(bool)` sends a `presence` frame, and again after each reconnect.
5. Events `DevicePresence` and `DeviceSignedOut`. `DeviceSignedOut` for this device signs the app
   out.

## Kotlin (`native/`)

1. `DeviceInfo` expect/actual:
   - Android: `Build.BRAND`, `Build.MODEL`, Android version;
   - iOS: "Apple", the device model and machine id, iOS version;
   - desktop: `os.name` / `os.version`, with the host name as the device name.
2. The engine sets the device on creation. `NomiApp` reports the lifecycle: resumed means
   foreground, stopped means background.

## Web (`frontend/`)

1. `hooks.server.ts` sets the `device_id` cookie if it's missing (UUID, two years). `handleFetch`
   sends `X-Device-Id`.
2. `ws-proxy.js` forwards the cookie as `x-device-id`, and `?tab=` as `x-tab-id`.
3. `userActivity.ts`:
   - keeps the tab id in sessionStorage;
   - sends `presence` on `visibilitychange`;
   - handles `DevicePresence` and `DeviceSignedOut` (own device → `/logout`).
4. The `(app)` layout registers the browser (name "Chrome on macOS", brand, model, OS from the user
   agent) at most once a day per browser.
5. `/account` gets a Devices section:
   - each device's name, kind, brand and model, OS, app version, status dot and last seen;
   - "This device" marked;
   - a Sign out action on the others;
   - live updates from `DevicePresence`.
