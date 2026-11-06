//! A publish's detail (asking Shipyard while it runs), a deployment's log, and how each kind of
//! Shipyard refusal reaches the person.

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn ok(data: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

fn refusal(status: u16, code: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({ "data": null, "error": { "code": code, "message": format!("{code} from Shipyard") } }))
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_publish_s_deployments_and_logs_come_from_shipyard(pool: PgPool) {
    let shipyard = MockServer::start().await;
    Mock::given(method("GET")).and(path_regex(r"/publishes/sp-1$")).respond_with(ok(json!({"status": "running", "deployments": [{"id": "d1", "status": "building"}, {"deployment_id": "d2", "service": "worker", "status": "queued"}, "d9", 42]}))).mount(&shipyard).await;
    Mock::given(method("GET")).and(path_regex(r"/publishes/sp-2$")).respond_with(ok(json!({"services": [{"deployment_id": "d3"}]}))).mount(&shipyard).await;
    Mock::given(method("GET")).and(path_regex(r"/deployments/d1$")).respond_with(ok(json!({"id": "d1", "steps": [{"name": "build", "status": "done"}], "log": "npm run build\nok"}))).mount(&shipyard).await;
    for (id, status, code) in [("busy", 409, "CONFLICT"), ("bad", 422, "INVALID"), ("gone", 404, "NOT_FOUND"), ("key", 401, "UNAUTHORIZED")] {
        Mock::given(method("GET")).and(path_regex(format!(r"/deployments/{id}$"))).respond_with(refusal(status, code)).mount(&shipyard).await;
    }
    Mock::given(method("GET")).and(path_regex(r"/deployments/down$")).respond_with(ResponseTemplate::new(503)).mount(&shipyard).await;

    let mut state = support::state(pool.clone());
    state.shipyard = Some(nomi_shipyard::ShipyardConfig { base_url: shipyard.uri(), key: "manage".into(), reveal_key: None, webhook_secret: None });
    let app = App::with_state(pool.clone(), state);
    let owner = app.person("owner@acme.test").await;
    let stranger = app.person("stranger@acme.test").await;
    let (project, _) = app.project(&owner, "Bakery").await;

    // Not published yet: there are no deployments to read.
    let deployment = |id: &str| format!("/api/projects/{project}/deploy/deployments/{id}");
    assert_eq!(app.get(&deployment("d1"), &owner).await.0, StatusCode::NOT_FOUND);

    sqlx::query("UPDATE projects SET shipyard_app = 'bakery' WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    let publish = |shipyard_id: &str| {
        let pool = pool.clone();
        let shipyard_id = shipyard_id.to_string();
        async move {
            sqlx::query_scalar::<_, Uuid>("INSERT INTO project_publishes (project_id, shipyard_publish_id, manifest) VALUES ($1, $2, '{}') RETURNING id")
                .bind(project)
                .bind(shipyard_id)
                .fetch_one(&pool)
                .await
                .unwrap()
        }
    };
    let running = publish("sp-1").await;
    let (status, detail) = app.get(&format!("/api/projects/{project}/deploy/publishes/{running}"), &owner).await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    let deployments = detail["deployments"].as_array().unwrap();
    assert_eq!(deployments.len(), 3, "a bare id is a deployment of the web service; other values are dropped");
    assert_eq!(deployments[2]["status"], "unknown");
    assert_eq!(deployments[0]["deployment_id"], "d1");
    assert_eq!(deployments[0]["service"], "web");
    assert_eq!(deployments[1]["id"], "d2");
    let other = publish("sp-2").await;
    let (_, detail) = app.get(&format!("/api/projects/{project}/deploy/publishes/{other}"), &owner).await;
    assert_eq!(detail["deployments"][0]["id"], "d3");
    assert_eq!(app.get(&format!("/api/projects/{project}/deploy/publishes/{}", Uuid::new_v4()), &owner).await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.get(&format!("/api/projects/{project}/deploy/publishes/{running}"), &stranger).await.0, StatusCode::NOT_FOUND);

    let (status, log) = app.get(&deployment("d1"), &owner).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(log["log"], "npm run build\nok");
    assert_eq!(app.get(&format!("{}?environment=Bad%20Env", deployment("d1")), &owner).await.0, StatusCode::BAD_REQUEST);
    for (id, expected) in [("busy", StatusCode::CONFLICT), ("bad", StatusCode::UNPROCESSABLE_ENTITY), ("gone", StatusCode::NOT_FOUND), ("key", StatusCode::BAD_GATEWAY), ("down", StatusCode::BAD_GATEWAY)] {
        assert_eq!(app.get(&deployment(id), &owner).await.0, expected, "{id}");
    }
}
