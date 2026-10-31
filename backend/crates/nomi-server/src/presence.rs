//! Which of a person's devices are on. Apps' presence connections publish on
//! `presence/{user_id}/{client_id}` (`nomi_realtime::presence`), with an `offline` last will the
//! broker sends when one drops. This keeps `device_connections` and `devices.presence` from what
//! the broker says, and tells the person's apps on their topic whenever a device changes. Several
//! servers may each run it: the change is written once, and only the server that wrote it tells.

use std::time::Duration;

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_realtime::{parse_presence_topic, user_topic, PresenceState, PresenceUpdate, UserEnvelope, PRESENCE_FILTER};

const MQTT_KEEP_ALIVE: Duration = Duration::from_secs(30);
const MQTT_CHANNEL_CAPACITY: usize = 64;
const RECONNECT_DELAY: Duration = Duration::from_secs(1);
const SHORT_RANDOM_LEN: usize = 8;

/// Sends to everything `user_id` has open (best effort, like the rest of realtime).
pub fn announce(user_id: Uuid, envelope: &UserEnvelope) {
    let Some(publisher) = crate::project_events::publisher() else { return };
    if let Err(e) = publisher.try_publish_to(&user_topic(user_id), envelope) {
        tracing::debug!(error = %e, %user_id, "couldn't tell the person's devices about a device");
    }
}

/// Listens to every presence topic for the life of the server, starting over when the broker goes
/// away (a new connection doesn't keep the old one's subscription).
pub async fn run(pool: PgPool, broker_host: String, broker_port: u16) {
    loop {
        let mut options = MqttOptions::new(format!("nomi-presence-{}", Uuid::new_v4()), broker_host.clone(), broker_port);
        options.set_keep_alive(MQTT_KEEP_ALIVE);
        let (client, mut eventloop) = AsyncClient::new(options, MQTT_CHANNEL_CAPACITY);
        if client.subscribe(PRESENCE_FILTER, QoS::AtLeastOnce).await.is_ok() {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        let Some((user_id, client_id)) = parse_presence_topic(&publish.topic) else { continue };
                        let Ok(update) = serde_json::from_slice::<PresenceUpdate>(&publish.payload) else { continue };
                        if let Err(e) = apply(&pool, user_id, client_id, &update).await {
                            tracing::warn!(error = %e, %user_id, "couldn't record a device's presence");
                        }
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(%error, "presence lost the broker, reconnecting");
                        break;
                    }
                }
            }
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

/// Records one connection's state, then the device's: online while any of its connections is in
/// front, away while all are in the background, offline when none is left.
pub async fn apply(pool: &PgPool, user_id: Uuid, client_id: &str, update: &PresenceUpdate) -> Result<(), sqlx::Error> {
    match update.state {
        PresenceState::Online | PresenceState::Away => {
            // Only a device the person registered: a stranger's id can't light up their list.
            sqlx::query(
                "INSERT INTO device_connections (client_id, device_id, connection, state) \
                 SELECT $1, $2, $3, $4 WHERE EXISTS (SELECT 1 FROM devices WHERE id = $2 AND user_id = $5) \
                 ON CONFLICT (client_id) DO UPDATE SET device_id = EXCLUDED.device_id, connection = EXCLUDED.connection, \
                   state = EXCLUDED.state, updated_at = now()",
            )
            .bind(client_id)
            .bind(update.device_id)
            .bind(update.connection)
            .bind(update.state.as_str())
            .bind(user_id)
            .execute(pool)
            .await?;
        }
        PresenceState::Offline => {
            // A replaced connection's goodbye leaves its replacement alone.
            sqlx::query("DELETE FROM device_connections WHERE client_id = $1 AND connection = $2")
                .bind(client_id)
                .bind(update.connection)
                .execute(pool)
                .await?;
        }
    }
    let changed: Option<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "UPDATE devices d SET presence = p.presence, last_seen_at = now() \
         FROM (SELECT CASE WHEN bool_or(state = 'online') THEN 'online' WHEN count(*) > 0 THEN 'away' ELSE 'offline' END AS presence \
               FROM device_connections WHERE device_id = $1) p \
         WHERE d.id = $1 AND d.user_id = $2 AND d.presence <> p.presence \
         RETURNING d.presence, d.last_seen_at",
    )
    .bind(update.device_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    if let Some((presence, last_seen_at)) = changed {
        let presence = match presence.as_str() {
            "online" => PresenceState::Online,
            "away" => PresenceState::Away,
            _ => PresenceState::Offline,
        };
        announce(user_id, &UserEnvelope::DevicePresence { device_id: update.device_id, presence, last_seen_at: last_seen_at.to_rfc3339() });
    }
    Ok(())
}

/// How one relay socket shows up on the broker: its client id, made from the device it serves,
/// and, for a presence connection, what it says about that device.
pub struct RelayIdentity {
    pub client_id: String,
    pub presence: Option<Speaker>,
}

/// A presence connection's voice: `online` when it opens, what the app says after, and the
/// `offline` last will the broker sends when it drops.
pub struct Speaker {
    user_id: Uuid,
    client_id: String,
    device_id: Uuid,
    connection: Uuid,
}

impl Speaker {
    pub async fn say(&self, client: &AsyncClient, state: PresenceState) {
        let update = PresenceUpdate { device_id: self.device_id, connection: self.connection, state };
        let payload = serde_json::to_vec(&update).expect("presence always serializes");
        if let Err(e) = client.publish(nomi_realtime::presence_topic(self.user_id, &self.client_id), QoS::AtLeastOnce, false, payload).await {
            tracing::debug!(error = %e, device_id = %self.device_id, "couldn't say a device's presence");
        }
    }
}

impl RelayIdentity {
    /// A relay that only forwards (a chat's, a project's): `<deviceId>-<purpose>-<random>`, since
    /// a device may have several open. Without a device id, `<purpose>-<random uuid>` as before.
    pub fn relay(headers: &axum::http::HeaderMap, purpose: &str) -> Self {
        let client_id = match crate::routes::devices::device_id(headers) {
            Some(device_id) => nomi_realtime::device_client_id(device_id, Some(&format!("{purpose}-{}", short_random()))),
            None => format!("{purpose}-{}", Uuid::new_v4()),
        };
        Self { client_id, presence: None }
    }

    /// The connection that says whether the device is on: the device id itself (a native app's
    /// one socket), or `<deviceId>-<tabId>` for a browser tab (`X-Tab-Id`). Without a device id
    /// it only forwards, like [`RelayIdentity::relay`].
    pub fn presence(headers: &axum::http::HeaderMap, user_id: Uuid, purpose: &str) -> Self {
        let Some(device_id) = crate::routes::devices::device_id(headers) else { return Self::relay(headers, purpose) };
        let tab = headers
            .get(crate::routes::devices::TAB_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .filter(|tab| nomi_realtime::valid_tab_id(tab));
        let client_id = nomi_realtime::device_client_id(device_id, tab);
        let speaker = Speaker { user_id, client_id: client_id.clone(), device_id, connection: Uuid::new_v4() };
        Self { client_id, presence: Some(speaker) }
    }

    /// Broker options for this socket: its client id and, for a presence connection, its last will.
    pub fn options(&self, broker_host: String, broker_port: u16) -> MqttOptions {
        let mut options = MqttOptions::new(self.client_id.clone(), broker_host, broker_port);
        options.set_keep_alive(MQTT_KEEP_ALIVE);
        if let Some(speaker) = &self.presence {
            options.set_last_will(nomi_realtime::offline_will(speaker.user_id, &speaker.client_id, speaker.device_id, speaker.connection));
        }
        options
    }
}

/// Eight hex characters: enough to tell one device's relays apart.
fn short_random() -> String {
    Uuid::new_v4().simple().to_string()[..SHORT_RANDOM_LEN].to_owned()
}
