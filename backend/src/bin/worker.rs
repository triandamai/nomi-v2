use std::env::var;
use std::time::Duration;

use nomi_orchestrator::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_from_settings_or_env};
use nomi_orchestrator::realtime::{MqttPublisher, StreamEnvelope};
use nomi_orchestrator::settings;
use nomi_orchestrator::turn::queue;

const NOTIFY_CHANNEL: &str = "turn_jobs_channel";
const POLL_FALLBACK_INTERVAL: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nomi_orchestrator=debug,info".into()),
        )
        .init();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let settings_key = settings::crypto::parse_key(
        &var("SETTINGS_ENCRYPTION_KEY").expect("SETTINGS_ENCRYPTION_KEY must be set"),
    )
    .expect("SETTINGS_ENCRYPTION_KEY must be 64 hex characters (32 bytes)");
    let mqtt_broker_host = var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_broker_port: u16 = var("MQTT_BROKER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1883);

    let pool = sqlx::PgPool::connect(&database_url).await.expect("failed to connect to database");
    tracing::info!("connected to database");
    sqlx::migrate!().run(&pool).await.expect("failed to run migrations");
    tracing::info!("migrations up to date");

    let http_client = reqwest::Client::new();
    let mqtt_client_id = format!("nomi-worker-{}", uuid::Uuid::new_v4());
    let mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &mqtt_client_id);

    let mut listener = sqlx::postgres::PgListener::connect(&database_url).await.expect("failed to connect listener");
    listener.listen(NOTIFY_CHANNEL).await.expect("failed to LISTEN on turn_jobs_channel");
    tracing::info!("listening for new turn jobs");

    loop {
        // Wake on NOTIFY, or on the fallback interval if a NOTIFY is ever missed — either way,
        // fall through to draining every currently-pending job before waiting again.
        let _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()).await;

        loop {
            let claimed = match queue::claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "failed to claim next turn job");
                    break;
                }
            };

            let provider = build_llm_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;
            let embedding_provider =
                build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            let user_id: Result<uuid::Uuid, sqlx::Error> =
                sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
                    .bind(claimed.sender_channel_identity_id)
                    .fetch_one(&pool)
                    .await;
            let user_id = match user_id {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!(error = %e, job_id = %claimed.id, "failed to resolve user_id for claimed job");
                    let _ = queue::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    continue;
                }
            };

            let result = nomi_orchestrator::turn::process_turn(
                &pool,
                &mqtt,
                provider.as_ref(),
                embedding_provider.as_ref(),
                claimed.id,
                claimed.session_id,
                claimed.sender_channel_identity_id,
                user_id,
                &claimed.text,
            )
            .await;

            match result {
                Ok(outcome) => {
                    let _ = queue::mark_completed(&pool, claimed.id).await;
                    // process_turn's TurnOutcome doesn't carry the persisted reply message's id
                    // (only its text) — the terminal envelope uses the turn_job_id as the
                    // correlation key; a future WebSocket bridge looks up the actual message row
                    // by session_id once it sees this event, rather than needing message_id here.
                    let _ = mqtt
                        .publish(
                            claimed.session_id,
                            &StreamEnvelope::TurnCompleted { turn_job_id: claimed.id, message_id: uuid::Uuid::nil() },
                        )
                        .await;
                    tracing::info!(job_id = %claimed.id, reply_len = outcome.reply.len(), "turn job completed");
                }
                Err(e) => {
                    // process_turn already published TurnFailed and recorded the agent_events row
                    // internally (see turn::process_turn) — this only updates the job's own status.
                    let _ = queue::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    tracing::warn!(job_id = %claimed.id, error = %e, "turn job failed");
                }
            }
        }
    }
}
