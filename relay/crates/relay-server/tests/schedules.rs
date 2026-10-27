//! Functions with a `schedule`: called with the service role at the minutes it names, once per
//! minute however often the scheduler ticks.
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use base64::Engine;
use chrono::{TimeZone, Utc};
use relay_control::backend::ensure_backend;
use relay_control::functions::{self, SourceFile};
use relay_control::Relay;
use relay_core::backend::FunctionSettings;
use relay_core::settings::FunctionsRuntime;
use relay_core::{Env, Tier};
use relay_server::schedules;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn a_schedule_is_due_at_the_minutes_it_names() {
    let at = |h, m| Utc.with_ymd_and_hms(2026, 10, 27, h, m, 0).unwrap();
    assert!(schedules::due("0 * * * *", at(14, 0)));
    assert!(!schedules::due("0 * * * *", at(14, 1)));
    assert!(schedules::due("*/15 * * * *", at(9, 45)));
    assert!(!schedules::due("not a schedule", at(9, 45)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_due_function_runs_once_with_the_service_role(pool: PgPool) {
    let runtime = MockServer::start().await;
    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true }))).mount(&runtime).await;
    Mock::given(method("POST")).and(path_regex("/nightly$")).respond_with(ResponseTemplate::new(200)).mount(&runtime).await;
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let uri = runtime.uri();
    let relay = Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".into()).with_settings(|s| s.functions = Some(FunctionsRuntime { url: uri, secret: "s".into() })));
    let project = Uuid::new_v4();
    ensure_backend(&relay, project, "Shop", Tier::Free, None).await.unwrap();
    let files = [("nightly", "0 3 * * *"), ("hello", "")].map(|(name, _)| SourceFile { path: format!("nomi/functions/{name}.ts"), content: "export default () => new Response()".into() });
    let settings = BTreeMap::from([("nightly".to_string(), FunctionSettings { schedule: Some("0 3 * * *".into()), ..FunctionSettings::default() })]);
    functions::deploy(&relay, project, Env::Production, &files, &settings).await.unwrap();

    let three = Utc.with_ymd_and_hms(2026, 10, 27, 3, 0, 0).unwrap();
    assert_eq!(schedules::tick(&relay, three).await, 1);
    assert_eq!(schedules::tick(&relay, three).await, 0, "already run this minute");
    assert_eq!(schedules::tick(&relay, three + chrono::Duration::minutes(1)).await, 0, "not due");

    let calls: Vec<_> = runtime.received_requests().await.unwrap().into_iter().filter(|r| r.url.path().ends_with("/nightly")).collect();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].url.path().contains("/production/1/"), "{}", calls[0].url);
    let claims: Value = serde_json::from_slice(&base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(calls[0].headers["x-nomi-claims"].to_str().unwrap()).unwrap()).unwrap();
    assert_eq!(claims["role"], "service");
    assert_eq!(calls[0].headers["x-nomi-schedule"], "0 3 * * *");
}
