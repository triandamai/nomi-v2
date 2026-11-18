//! Admin → Projects and Sandboxes: every project with its sandbox and issues at a glance, every
//! sandbox record in every state, and a project's issues for an admin who isn't in it.

mod support;

use axum::http::StatusCode;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

async fn sandbox(pool: &PgPool, project_id: Uuid, sandbox_id: &str, state: &str) {
    sqlx::query(
        "INSERT INTO project_sandboxes (project_id, provider, sandbox_id, state) VALUES ($1, 'boat', $2, $3) \
         ON CONFLICT (project_id) DO UPDATE SET sandbox_id = EXCLUDED.sandbox_id, state = EXCLUDED.state",
    )
    .bind(project_id)
    .bind(sandbox_id)
    .bind(state)
    .execute(pool)
    .await
    .unwrap();
}

async fn issue(pool: &PgPool, project_id: Uuid, number: i32, title: &str, status: &str) {
    sqlx::query("INSERT INTO project_issues (project_id, number, title, status, kind, severity) VALUES ($1, $2, $3, $4, 'bug', 'high')")
        .bind(project_id)
        .bind(number)
        .bind(title)
        .bind(status)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_an_admin_sees_the_projects(pool: PgPool) {
    let app = App::new(pool.clone());
    let budi = app.person("budi@projects.test").await;
    for path in ["/api/admin/projects", "/api/admin/sandboxes", &format!("/api/admin/projects/{}/issues", Uuid::new_v4())] {
        assert_eq!(app.get(path, &budi).await.0, StatusCode::FORBIDDEN, "{path}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_project_shows_its_sandbox_and_issues(pool: PgPool) {
    let app = App::new(pool.clone());
    let admin = app.admin("admin@projects.test").await;
    let owner = app.person("budi@projects.test").await;
    let (bakery, _) = app.project(&owner, "roti-pagi").await;
    let (quiet, _) = app.project(&owner, "warung-kopi").await;
    sandbox(&pool, bakery, "box-1", "stopped").await;
    sandbox(&pool, bakery, "box-2", "running").await;
    issue(&pool, bakery, 1, "Checkout button does nothing", "open").await;
    issue(&pool, bakery, 2, "Typo on the menu", "closed").await;

    let (status, body) = app.get("/api/admin/projects", &admin).await;
    assert_eq!(status, StatusCode::OK);
    let row = |id: Uuid| body["projects"].as_array().unwrap().iter().find(|p| p["id"] == id.to_string()).cloned().unwrap();
    let bakery_row = row(bakery);
    assert_eq!(bakery_row["name"], "roti-pagi");
    assert_eq!(bakery_row["owner_email"], "budi@projects.test");
    assert_eq!(bakery_row["sandbox_phase"], "running");
    assert_eq!(bakery_row["sandbox_records"], 2);
    assert_eq!((bakery_row["open_issues"].as_i64(), bakery_row["issues"].as_i64()), (Some(1), Some(2)));
    let quiet_row = row(quiet);
    assert!(quiet_row["sandbox_phase"].is_null(), "no sandbox yet");
    assert_eq!(quiet_row["sandbox_records"], 0);

    let (_, found) = app.get("/api/admin/projects?q=kopi", &admin).await;
    assert_eq!(found["projects"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn every_sandbox_record_is_listed_whatever_its_state(pool: PgPool) {
    let app = App::new(pool.clone());
    let admin = app.admin("admin@projects.test").await;
    let owner = app.person("budi@projects.test").await;
    let (bakery, _) = app.project(&owner, "roti-pagi").await;
    let (other, _) = app.project(&owner, "warung-kopi").await;
    sandbox(&pool, bakery, "box-1", "stopped").await;
    sandbox(&pool, bakery, "box-2", "starting").await;
    sandbox(&pool, other, "box-9", "running").await;

    let (status, body) = app.get(&format!("/api/admin/sandboxes?project={bakery}"), &admin).await;
    assert_eq!(status, StatusCode::OK);
    let rows = body["sandboxes"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "only that project's");
    assert_eq!((rows[0]["sandbox_id"].as_str(), rows[0]["phase"].as_str(), rows[0]["current"].as_bool()), (Some("box-2"), Some("waking"), Some(true)));
    assert_eq!((rows[1]["sandbox_id"].as_str(), rows[1]["phase"].as_str(), rows[1]["current"].as_bool()), (Some("box-1"), Some("stopped"), Some(false)));
    assert_eq!(rows[0]["project_name"], "roti-pagi");

    let (_, all) = app.get("/api/admin/sandboxes", &admin).await;
    assert_eq!(all["sandboxes"].as_array().unwrap().len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_admin_reads_a_projects_issues_without_being_in_it(pool: PgPool) {
    let app = App::new(pool.clone());
    let admin = app.admin("admin@projects.test").await;
    let owner = app.person("budi@projects.test").await;
    let (bakery, _) = app.project(&owner, "roti-pagi").await;
    issue(&pool, bakery, 1, "Checkout button does nothing", "open").await;
    issue(&pool, bakery, 2, "Typo on the menu", "closed").await;

    let (status, body) = app.get(&format!("/api/admin/projects/{bakery}/issues"), &admin).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["project"]["name"], "roti-pagi");
    let titles: Vec<&str> = body["issues"].as_array().unwrap().iter().map(|i| i["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["Typo on the menu", "Checkout button does nothing"], "newest first, open and closed");
    assert_eq!(app.get(&format!("/api/admin/projects/{}/issues", Uuid::new_v4()), &admin).await.0, StatusCode::NOT_FOUND);
}
