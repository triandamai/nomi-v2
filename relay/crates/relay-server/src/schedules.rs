//! Functions with a `schedule` in `backend.json`: every minute, each one that's due is called with
//! the service role. Each run is claimed in `relay.function_runs` first, so two Relay instances
//! never both run it.

use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::http::{HeaderMap, HeaderValue, Method};
use chrono::{DateTime, Timelike, Utc};
use relay_control::backend::backend_keys;
use relay_control::functions::CurrentBuild;
use relay_control::Relay;
use relay_core::backend::FunctionSettings;
use relay_core::Env;
use relay_gateway::auth::{Caller, Kind};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::functions_routes::{invoke, runtime, Call};
use crate::gateway::Gateway;

const SCHEDULE_HEADER: &str = "x-nomi-schedule";
const MINUTE: Duration = Duration::from_secs(60);
const RUN_CLAIMS_KEPT_DAYS: i64 = 1;

/// Whether `schedule` names `minute` (UTC).
pub fn due(schedule: &str, minute: DateTime<Utc>) -> bool {
    croner::Cron::new(schedule).parse().ok().and_then(|cron| cron.is_time_matching(&minute).ok()).unwrap_or(false)
}

/// Runs what's due at `minute` and answers how many runs this call claimed.
pub async fn tick(relay: &Arc<Relay>, minute: DateTime<Utc>) -> usize {
    let gateway = Gateway { relay: relay.clone(), http: reqwest::Client::new(), hub: Default::default() };
    let Some(runtime) = runtime(&gateway) else { return 0 };
    let minute = minute.with_second(0).and_then(|m| m.with_nanosecond(0)).unwrap_or(minute);
    let builds: Vec<(Uuid, String, i32, Value)> = match sqlx::query_as("SELECT project_id, env, version, functions FROM relay.function_builds WHERE status = 'current'")
        .fetch_all(&relay.state)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, "couldn't read scheduled functions");
            return 0;
        }
    };
    // Claims only need to outlive the minute they're for.
    let _ = sqlx::query("DELETE FROM relay.function_runs WHERE minute < $1").bind(minute - chrono::Duration::days(RUN_CLAIMS_KEPT_DAYS)).execute(&relay.state).await;
    let mut claimed = 0;
    for (project, env_name, version, functions) in builds {
        let (Some(env), Ok(functions)) = (Env::parse(&env_name), serde_json::from_value::<BTreeMap<String, FunctionSettings>>(functions)) else { continue };
        for (name, settings) in &functions {
            let Some(schedule) = settings.schedule.as_deref().filter(|s| due(s, minute)) else { continue };
            let won = sqlx::query("INSERT INTO relay.function_runs (project_id, env, name, minute) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING")
                .bind(project)
                .bind(env.as_str())
                .bind(name)
                .bind(minute)
                .execute(&relay.state)
                .await
                .map(|done| done.rows_affected() == 1)
                .unwrap_or(false);
            if !won {
                continue;
            }
            claimed += 1;
            let mut headers = HeaderMap::new();
            if let Ok(keys) = backend_keys(relay, project).await {
                if let Some(key) = keys.environments.get(env.as_str()).and_then(|k| HeaderValue::from_str(&format!("Bearer {}", k.service_key)).ok()) {
                    headers.insert(axum::http::header::AUTHORIZATION, key);
                }
            }
            if let Ok(value) = HeaderValue::from_str(schedule) {
                headers.insert(SCHEDULE_HEADER, value);
            }
            let build = CurrentBuild { version, functions: functions.clone() };
            let caller = Caller { kind: Kind::Service, sub: None, email: None, sid: None };
            let call = Call { name: name.clone(), rest: String::new(), query: None, method: Method::POST, headers, body: Bytes::new(), caller };
            let answer = invoke(&gateway, &runtime, project, env, &build, call).await;
            if !answer.status().is_success() {
                tracing::warn!(%project, env = env.as_str(), function = %name, status = %answer.status(), "scheduled function failed");
            }
        }
    }
    claimed
}

/// Ticks at the start of every minute, for good.
pub fn start(relay: Arc<Relay>) {
    tokio::spawn(async move {
        loop {
            let now = Utc::now();
            let wait = MINUTE.saturating_sub(Duration::from_secs(u64::from(now.second())) + Duration::from_nanos(u64::from(now.nanosecond())));
            tokio::time::sleep(wait).await;
            let relay = relay.clone();
            // Each minute on its own, so a slow function doesn't push the next minute back.
            tokio::spawn(async move { tick(&relay, Utc::now()).await });
        }
    });
}
