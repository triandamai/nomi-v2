use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nomi_agent_core::content_block::ContentBlock;
use nomi_agent_core::SubAgent;
use nomi_agent_health::connection::{self, ConnectError, HEVY};
use nomi_agent_health::hevy::{Hevy, HevyError};
use nomi_agent_health::HealthAgent;

const KEY: [u8; 32] = [7; 32];

async fn person(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn hevy_at<'a>(http: &'a reqwest::Client, server: &MockServer, key: &'a str) -> Hevy<'a> {
    Hevy { http, base_url: server.uri(), api_key: key }
}

#[tokio::test]
async fn every_call_carries_the_key_and_a_refused_key_reads_as_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/user/info"))
        .and(header("api-key", "good"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"id": "u1", "name": "ana", "url": "https://hevy.com/user/ana"}})))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/v1/user/info")).respond_with(ResponseTemplate::new(401)).mount(&server).await;
    let http = reqwest::Client::new();
    assert_eq!(hevy_at(&http, &server, "good").user_info().await.unwrap()["name"], "ana");
    assert_eq!(hevy_at(&http, &server, "bad").user_info().await.unwrap_err(), HevyError::KeyRejected);
}

#[tokio::test]
async fn a_measurement_for_a_logged_day_updates_it_and_keeps_the_rest() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/body_measurements"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({"error": "Body measurement already exists for this date"})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/body_measurements/2026-10-12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"date": "2026-10-12", "weight_kg": 80.0, "waist": 82.0})))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/body_measurements/2026-10-12"))
        .and(body_partial_json(json!({"weight_kg": 79.4, "waist": 82.0})))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    let http = reqwest::Client::new();
    let values = json!({"weight_kg": 79.4}).as_object().unwrap().clone();
    hevy_at(&http, &server, "k").log_body_measurement("2026-10-12", &values).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_key_is_checked_before_it_is_saved_and_kept_encrypted(pool: PgPool) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/user/info"))
        .and(header("api-key", "hevy-key-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"id": "u1", "name": "ana", "url": ""}})))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/v1/user/info")).respond_with(ResponseTemplate::new(401)).mount(&server).await;

    let ana = person(&pool).await;
    let http = reqwest::Client::new();
    let mut conn = pool.acquire().await.unwrap();
    assert!(matches!(connection::connect_hevy(&mut conn, &http, &server.uri(), &KEY, ana, "wrong").await, Err(ConnectError::Rejected(HevyError::KeyRejected))));
    assert!(connection::status(&mut conn, ana, HEVY).await.unwrap().is_none(), "a refused key isn't saved");

    let saved = connection::connect_hevy(&mut conn, &http, &server.uri(), &KEY, ana, " hevy-key-123 ").await.unwrap();
    assert_eq!(saved.account_name.as_deref(), Some("ana"));
    let stored: Vec<u8> = sqlx::query_scalar("SELECT api_key_encrypted FROM health_connections").fetch_one(&pool).await.unwrap();
    assert!(!String::from_utf8_lossy(&stored).contains("hevy-key-123"));
    assert_eq!(connection::api_key(&mut conn, &KEY, ana, HEVY).await.unwrap().as_deref(), Some("hevy-key-123"));
    assert!(connection::status(&mut conn, ana, HEVY).await.unwrap().unwrap().last_used_at.is_some());

    // Someone else has no access to it.
    let budi = person(&pool).await;
    assert_eq!(connection::api_key(&mut conn, &KEY, budi, HEVY).await.unwrap(), None);
    assert!(connection::disconnect(&mut conn, ana, HEVY).await.unwrap());
    assert!(connection::status(&mut conn, ana, HEVY).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn tools_answer_with_short_text_or_a_connect_card(pool: PgPool) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/user/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"id": "u1", "name": "ana", "url": ""}})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/workouts"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"page": 1, "page_count": 3, "workouts": [{
            "id": "w1", "title": "Leg day", "start_time": "2026-10-11T06:00:00Z", "end_time": "2026-10-11T07:00:00Z",
            "exercises": [{"title": "Squat (Barbell)", "exercise_template_id": "D04AC939", "sets": [
                {"type": "normal", "weight_kg": 100, "reps": 5}, {"type": "normal", "weight_kg": 100, "reps": 5}
            ]}]
        }]})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/workouts"))
        .and(body_partial_json(json!({"workout": {"title": "Morning run", "exercises": [{"exercise_template_id": "AC1BB830"}]}})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "w2", "title": "Morning run", "start_time": "2026-10-12T06:00:00Z", "end_time": "2026-10-12T06:30:00Z",
            "exercises": [{"title": "Running", "exercise_template_id": "AC1BB830", "sets": [{"type": "normal", "distance_meters": 5000, "duration_seconds": 1800}]}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let agent = HealthAgent::new(reqwest::Client::new(), KEY).with_hevy_base_url(server.uri());
    let ana = person(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let (session, agent_session) = (Uuid::new_v4(), Uuid::new_v4());

    // Not connected yet: a card to connect Hevy.
    let outcome = agent.execute_tool(&mut conn, session, agent_session, ana, "hevy_workouts", json!({})).await.unwrap();
    assert!(matches!(outcome.block, Some(ContentBlock::ConnectorConnect { ref connector, ref reason }) if connector == "hevy" && reason == "not_connected"));

    connection::connect_hevy(&mut conn, &reqwest::Client::new(), &server.uri(), &KEY, ana, "k").await.unwrap();
    let outcome = agent.execute_tool(&mut conn, session, agent_session, ana, "hevy_workouts", json!({})).await.unwrap();
    assert_eq!(outcome.display_text, "Leg day (id w1) — 2026-10-11 06:00, 60 min\n  - Squat (Barbell) [D04AC939]: 2×(100kg×5)\n(page 1 of 3)");

    let logged = agent
        .execute_tool(&mut conn, session, agent_session, ana, "hevy_log_workout", json!({
            "title": "Morning run", "start_time": "2026-10-12T06:00:00Z", "end_time": "2026-10-12T06:30:00Z", "is_private": false,
            "exercises": [{"exercise_template_id": "AC1BB830", "sets": [{"type": "normal", "distance_meters": 5000, "duration_seconds": 1800}]}],
            "ignored": "not sent"
        }))
        .await
        .unwrap();
    assert!(logged.display_text.starts_with("Logged.\nMorning run (id w2)"), "{}", logged.display_text);

    // Every tool is the agent's own and none waits for approval (it's the person's own log).
    for tool in agent.tools() {
        assert!(tool.name.starts_with("hevy_"), "{}", tool.name);
        assert!(!agent.tool_needs_approval(&tool.name));
    }
}
