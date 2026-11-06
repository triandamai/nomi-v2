//! Every Hevy tool through the agent, what each answers in text, the cards for saved workouts and
//! routines, and how Hevy's errors read.

use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nomi_agent_core::content_block::{ContentBlock, ToolOutcome};
use nomi_agent_core::SubAgent;
use nomi_agent_health::connection;
use nomi_agent_health::hevy::{Hevy, HevyError};
use nomi_agent_health::HealthAgent;

const KEY: [u8; 32] = [5; 32];

async fn hevy(server: &MockServer, verb: &str, at: &str, status: u16, body: Value) {
    Mock::given(method(verb)).and(path(at)).respond_with(ResponseTemplate::new(status).set_body_json(body)).mount(server).await;
}

struct Setup {
    pool: PgPool,
    agent: HealthAgent,
    user: Uuid,
}

impl Setup {
    async fn new(pool: PgPool, server: &MockServer) -> Self {
        hevy(server, "GET", "/v1/user/info", 200, json!({"data": {"id": "u1", "name": "ana", "url": ""}})).await;
        let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
        let mut conn = pool.acquire().await.unwrap();
        connection::connect_hevy(&mut conn, &reqwest::Client::new(), &server.uri(), &KEY, user, "k").await.unwrap();
        Setup { pool, agent: HealthAgent::new(reqwest::Client::new(), KEY).with_hevy_base_url(server.uri()), user }
    }

    async fn run(&self, tool: &str, input: Value) -> Result<ToolOutcome, String> {
        let mut conn = self.pool.acquire().await.unwrap();
        self.agent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), self.user, tool, input).await
    }

    async fn text(&self, tool: &str, input: Value) -> String {
        self.run(tool, input).await.unwrap().display_text
    }
}

const WORKOUT: &str = r#"{"id": "w1", "title": "Pull", "start_time": "2026-10-11T06:00:00Z", "end_time": "bad",
  "exercises": [{"title": "Row", "exercise_template_id": "R1", "notes": "slow", "superset_id": 2, "sets": [
    {"type": "dropset", "weight_kg": 50, "reps": 10}, {"reps": 12}, {"weight_kg": 20}, {"distance_meters": 400}, {"duration_seconds": 45}, {}
  ]}]}"#;

#[sqlx::test(migrations = "../../migrations")]
async fn workouts_read_count_change_log_and_update(pool: PgPool) {
    let server = MockServer::start().await;
    let s = Setup::new(pool, &server).await;
    let workout: Value = serde_json::from_str(WORKOUT).unwrap();

    hevy(&server, "GET", "/v1/workouts/w1", 200, workout.clone()).await;
    let one = s.text("hevy_workout", json!({"workout_id": "w1"})).await;
    assert!(one.contains("50kg×10 (dropset), ×12, 20kg, 400m, 45s, set"), "{one}");
    assert!(one.contains("; superset 2; notes: slow"), "{one}");

    hevy(&server, "GET", "/v1/workouts/count", 200, json!({"workout_count": 42})).await;
    assert_eq!(s.text("hevy_workout_count", json!({})).await, "42 workouts logged.");

    Mock::given(method("GET"))
        .and(path("/v1/workouts/events"))
        .and(query_param("since", "2026-10-01T00:00:00Z"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"page": 2, "page_count": 2, "events": [
            {"type": "deleted", "id": "w9", "deleted_at": "2026-10-05T10:00:00Z"},
            {"type": "updated", "workout": workout}
        ]})))
        .mount(&server)
        .await;
    let changes = s.text("hevy_workout_changes", json!({"since": "2026-10-01T00:00:00Z", "page": 2})).await;
    assert!(changes.starts_with("Deleted workout w9 at 2026-10-05 10:00\n\nUpdated: Pull (id w1)"), "{changes}");

    Mock::given(method("PUT"))
        .and(path("/v1/workouts/w1"))
        .and(body_partial_json(json!({"workout": {"title": "Pull (heavy)"}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "w1", "title": "Pull (heavy)", "exercises": []})))
        .mount(&server)
        .await;
    let updated = s.run("hevy_update_workout", json!({"workout_id": "w1", "title": "Pull (heavy)"})).await.unwrap();
    assert!(updated.display_text.starts_with("Updated.\nPull (heavy) (id w1)"));
    assert!(matches!(updated.block, Some(ContentBlock::View(_))), "a saved workout shows as a card");

    // Nothing logged on an empty page.
    let empty = MockServer::start().await;
    let s2 = Setup::new(s.pool.clone(), &empty).await;
    hevy(&empty, "GET", "/v1/workouts", 200, json!({"workouts": []})).await;
    hevy(&empty, "GET", "/v1/workouts/events", 200, json!({"events": []})).await;
    hevy(&empty, "GET", "/v1/routines", 200, json!({"routines": []})).await;
    hevy(&empty, "GET", "/v1/routine_folders", 200, json!({})).await;
    hevy(&empty, "GET", "/v1/body_measurements", 200, json!({"body_measurements": []})).await;
    assert_eq!(s2.text("hevy_workouts", json!({})).await, "No workouts on this page.");
    assert_eq!(s2.text("hevy_workout_changes", json!({"since": "x"})).await, "No changes since then.");
    assert_eq!(s2.text("hevy_routines", json!({})).await, "No routines on this page.");
    assert!(s2.text("hevy_routine_folders", json!({})).await.starts_with("No routine folders"));
    assert_eq!(s2.text("hevy_body_measurements", json!({})).await, "No body measurements logged.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn routines_and_folders(pool: PgPool) {
    let server = MockServer::start().await;
    let s = Setup::new(pool, &server).await;
    let routine = json!({"id": "r1", "title": "Upper A", "folder_id": 7, "notes": "Twice a week", "exercises": [
        {"title": "Bench", "exercise_template_id": "B1", "rest_seconds": 90, "sets": [{"weight_kg": 60, "rep_range": {"start": 8, "end": 10}}]}
    ]});
    hevy(&server, "GET", "/v1/routines", 200, json!({"page": 1, "page_count": 1, "routines": [routine.clone()]})).await;
    let list = s.text("hevy_routines", json!({"page": 0})).await;
    assert!(list.contains("Upper A (id r1), folder 7\n  Twice a week"), "{list}");
    assert!(list.contains("60kg 8-10 reps; rest 90s"), "{list}");

    hevy(&server, "GET", "/v1/routines/r1", 200, json!({"routine": routine.clone()})).await;
    assert!(s.text("hevy_routine", json!({"routine_id": "r1"})).await.starts_with("Upper A"));

    Mock::given(method("POST"))
        .and(path("/v1/routines"))
        .and(body_partial_json(json!({"routine": {"title": "Legs", "folder_id": null}})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"routine": {"id": "r2", "title": "Legs", "exercises": []}})))
        .mount(&server)
        .await;
    let created = s.run("hevy_create_routine", json!({"title": "Legs", "exercises": []})).await.unwrap();
    assert!(created.display_text.starts_with("Created.\nLegs (id r2)"));
    assert!(created.block.is_some());

    hevy(&server, "PUT", "/v1/routines/r2", 200, json!({"routine": {"id": "r2", "title": "Legs B"}})).await;
    let updated = s.run("hevy_update_routine", json!({"routine_id": "r2", "title": "Legs B"})).await.unwrap();
    assert!(updated.display_text.starts_with("Updated.\nLegs B"));
    assert!(updated.block.is_none());

    hevy(&server, "GET", "/v1/routine_folders", 200, json!({"page": 1, "page_count": 1, "routine_folders": [{"id": 7, "title": "Gym"}]})).await;
    assert!(s.text("hevy_routine_folders", json!({})).await.starts_with("Gym (id 7)"));
    hevy(&server, "POST", "/v1/routine_folders", 201, json!({"routine_folder": {"id": 8, "title": "Home"}})).await;
    assert_eq!(s.text("hevy_create_routine_folder", json!({"title": "Home"})).await, "Created folder Home (id 8).");
}

#[sqlx::test(migrations = "../../migrations")]
async fn exercises_are_found_across_pages_and_created(pool: PgPool) {
    let server = MockServer::start().await;
    let s = Setup::new(pool, &server).await;
    let squat = json!({"id": "S1", "title": "Squat (Barbell)", "type": "weight_reps", "primary_muscle_group": "quadriceps"});
    let curl = json!({"id": "C1", "title": "Curl", "type": "weight_reps", "primary_muscle_group": "biceps", "is_custom": true});
    Mock::given(method("GET")).and(path("/v1/exercise_templates")).and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"page": 1, "page_count": 2, "exercise_templates": [squat]})))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/v1/exercise_templates")).and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"page": 2, "page_count": 2, "exercise_templates": [curl]})))
        .expect(2)
        .mount(&server)
        .await;
    let found = s.text("hevy_find_exercises", json!({"query": "curl"})).await;
    assert_eq!(found, "Curl — id C1 (weight_reps, biceps, custom)");
    // The list is kept: a second search asks Hevy nothing.
    assert!(s.text("hevy_find_exercises", json!({"muscle_group": "quadriceps"})).await.contains("id S1"));

    hevy(&server, "POST", "/v1/exercise_templates", 200, json!({"id": "N1"})).await;
    assert_eq!(s.text("hevy_create_exercise", json!({"title": "Sled push", "exercise_type": "distance_duration"})).await, "Created custom exercise Sled push (id N1).");
    // Creating one forgets the kept list, so the next search asks Hevy again.
    s.text("hevy_find_exercises", json!({"custom_only": true})).await;
    let first_pages = server.received_requests().await.unwrap().iter().filter(|r| r.url.path() == "/v1/exercise_templates" && r.url.query().unwrap_or("").contains("page=1")).count();
    assert_eq!(first_pages, 2);

    Mock::given(method("GET"))
        .and(path("/v1/exercise_history/S1"))
        .and(query_param("start_date", "2026-01-01"))
        .and(query_param("end_date", "2026-12-31"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"exercise_history": []})))
        .mount(&server)
        .await;
    assert!(s.text("hevy_exercise_history", json!({"exercise_template_id": "S1", "start_date": "2026-01-01", "end_date": "2026-12-31"})).await.starts_with("No sets"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn body_measurements_are_listed_and_logged(pool: PgPool) {
    let server = MockServer::start().await;
    let s = Setup::new(pool, &server).await;
    hevy(&server, "GET", "/v1/body_measurements", 200, json!({"page": 1, "page_count": 1, "body_measurements": [{"date": "2026-10-12", "weight_kg": 79.4, "waist": null}]})).await;
    assert_eq!(s.text("hevy_body_measurements", json!({})).await, "2026-10-12: weight_kg 79.4\n(page 1 of 1)");

    hevy(&server, "POST", "/v1/body_measurements", 201, json!({})).await;
    assert_eq!(s.text("hevy_log_body_measurement", json!({"date": "2026-10-13", "weight_kg": 79.0, "note": "ignored"})).await, "Logged for 2026-10-13: weight_kg 79.0.");
    assert_eq!(s.run("hevy_log_body_measurement", json!({"date": "2026-10-13", "weight_kg": "heavy"})).await.unwrap_err(), "Hevy refused it: give at least one measurement");
    assert_eq!(s.run("hevy_log_body_measurement", json!({"weight_kg": 79})).await.unwrap_err(), "Hevy refused it: missing date");
}

#[sqlx::test(migrations = "../../migrations")]
async fn hevy_errors_read_plainly_and_a_rejected_key_asks_for_a_new_one(pool: PgPool) {
    let server = MockServer::start().await;
    let s = Setup::new(pool, &server).await;
    hevy(&server, "GET", "/v1/workouts/gone", 404, json!({})).await;
    hevy(&server, "GET", "/v1/routines/limit", 403, json!({"error": "routine limit reached"})).await;
    hevy(&server, "GET", "/v1/routines/forbidden", 403, json!({})).await;
    hevy(&server, "POST", "/v1/routine_folders", 409, json!({"error": "exists"})).await;
    hevy(&server, "GET", "/v1/exercise_templates/bad", 400, json!({"error": "bad id"})).await;
    Mock::given(method("GET")).and(path("/v1/exercise_templates/plain")).respond_with(ResponseTemplate::new(422).set_body_string("unprocessable")).mount(&server).await;
    hevy(&server, "GET", "/v1/workouts/count", 502, json!({})).await;

    assert_eq!(s.run("hevy_workout", json!({"workout_id": "gone"})).await.unwrap_err(), "Hevy has no such workout");
    assert_eq!(s.run("hevy_routine", json!({"routine_id": "limit"})).await.unwrap_err(), "the Hevy account has reached a limit: routine limit reached");
    assert_eq!(s.run("hevy_create_routine_folder", json!({"title": "Gym"})).await.unwrap_err(), "Hevy already has that: exists");
    assert_eq!(s.run("hevy_workout_count", json!({})).await.unwrap_err(), "couldn't reach Hevy: Hevy returned 502");
    assert_eq!(s.run("hevy_lift", json!({})).await.unwrap_err(), "Hevy refused it: unknown tool hevy_lift");

    let rejected = s.run("hevy_routine", json!({"routine_id": "forbidden"})).await.unwrap();
    assert!(matches!(rejected.block, Some(ContentBlock::ConnectorConnect { ref reason, .. }) if reason == "key_rejected"));

    let http = reqwest::Client::new();
    let client = Hevy { http: &http, base_url: format!("{}/", server.uri()), api_key: "k" };
    assert_eq!(client.exercise_template("bad").await.unwrap_err(), HevyError::Rejected("bad id".into()));
    assert_eq!(client.exercise_template("plain").await.unwrap_err(), HevyError::Rejected("unprocessable".into()));
    // Ids are cut down to safe characters before they go in a path.
    assert_eq!(client.workout("../gone").await.unwrap_err(), HevyError::NotFound("workout".into()));
    let unreachable = Hevy { http: &http, base_url: "http://127.0.0.1:9".into(), api_key: "k" };
    let error = unreachable.user_info().await.unwrap_err();
    assert!(matches!(error, HevyError::Unreachable(_)));
    assert!(error.to_string().starts_with("couldn't reach Hevy"));
    assert!(HevyError::KeyRejected.to_string().contains("rejected the API key"));
}

#[test]
fn the_agent_describes_itself() {
    let agent = HealthAgent::new(reqwest::Client::new(), KEY);
    assert_eq!(agent.agent_type(), "health");
    assert!(!agent.system_prompt().is_empty());
    assert!(!agent.intent_label().is_empty() && !agent.intent_description().is_empty());
    assert!(agent.uses_memory() && agent.uses_personality() && agent.wants_current_time());
    assert!(agent.max_tool_turns() > 0);
    let _ = HealthAgent::from_env();
    assert_eq!(nomi_agent_health::hevy::base_url().is_empty(), false);
}
