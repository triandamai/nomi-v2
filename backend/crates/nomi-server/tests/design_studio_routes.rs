//! The Design Studio's routes: a chat's previews listed for the people who can open the chat
//! (where Design is rolled out), and a version's files served to the Studio's sandboxed frame
//! by an unguessable name, with no sign-in and nothing of Nomi's within reach.

mod support;

use std::collections::BTreeMap;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use nomi_agent_core::design_artifact::{self, DesignArtifact};
use serde_json::json;
use sqlx::PgPool;
use support::{App, Person};
use uuid::Uuid;

/// The `design` setting is read through the process-wide settings cache and pool, so the tests
/// that set it take turns.
static GLOBALS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn design_on(pool: &PgPool, admin: Uuid, on: bool) {
    std::env::set_var("SETTINGS_ENCRYPTION_KEY", hex::encode(nomi_test_support::TEST_SETTINGS_KEY));
    nomi_settings::config::install(pool.clone());
    let pairs: &[(&str, &str)] = if on { &[("enabled", "on"), ("rollout", "all")] } else { &[("enabled", "off")] };
    let values: BTreeMap<String, String> = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    nomi_settings::config::save(pool, &nomi_test_support::TEST_SETTINGS_KEY, &nomi_settings::config::DESIGN, &values, admin).await.unwrap();
    nomi_settings::config::forget();
}

async fn chat_of(pool: &PgPool, who: &Person) -> Uuid {
    sqlx::query_scalar("INSERT INTO sessions (org_id, user_id, channel, chat_id) SELECT org_id, $1, 'web', $2 FROM memberships WHERE user_id = $1 LIMIT 1 RETURNING id")
        .bind(who.id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap()
}

fn preview(direction: &str, heading: &str) -> DesignArtifact {
    let html = format!(
        "<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Home</title><link rel=\"stylesheet\" href=\"styles.css\"></head>\
         <body><h1 data-design-id=\"hero\">{heading}</h1></body></html>"
    );
    let menu = "<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Menu</title></head><body><ul data-design-id=\"menu-list\"></ul></body></html>";
    DesignArtifact::parse(&json!({
        "direction_id": direction,
        "title": heading,
        "files": [{"path": "index.html", "content": html}, {"path": "styles.css", "content": "h1 { margin: 0 }"}, {"path": "menu/index.html", "content": menu}],
        "elements": [{"id": "hero", "kind": "section", "label": "Hero"}]
    }))
    .unwrap()
}

async fn save(pool: &PgPool, session_id: Uuid, who: &Person, artifact: &DesignArtifact) {
    let mut conn = pool.acquire().await.unwrap();
    design_artifact::save(&mut conn, session_id, Uuid::new_v4(), who.id, None, artifact).await.unwrap();
}

async fn fetch(app: &App, uri: &str) -> (StatusCode, axum::http::HeaderMap, String) {
    let (status, headers, body) = app.raw(Request::builder().uri(uri).body(Body::empty()).unwrap()).await;
    (status, headers, String::from_utf8_lossy(&body).into_owned())
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_studio_lists_a_chats_previews_for_the_people_who_can_open_it(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let admin = app.admin("admin@acme.test").await;
    let ada = app.person("ada@acme.test").await;
    let bob = app.person("bob@acme.test").await;
    let session_id = chat_of(&pool, &ada).await;
    save(&pool, session_id, &ada, &preview("editorial", "Fresh at six")).await;
    save(&pool, session_id, &ada, &preview("swiss", "Bread, daily")).await;
    let uri = format!("/api/sessions/{session_id}/design");

    design_on(&pool, admin.id, false).await;
    assert_eq!(app.get(&uri, &ada).await.0, StatusCode::NOT_FOUND, "the Studio isn't there until Design is rolled out");

    design_on(&pool, admin.id, true).await;
    let (status, body) = app.get(&uri, &ada).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let previews = body["previews"].as_array().unwrap();
    assert_eq!(previews.iter().map(|p| (p["version"].as_i64().unwrap(), p["direction_id"].as_str().unwrap())).collect::<Vec<_>>(), [(1, "editorial"), (2, "swiss")]);
    let newest = &previews[1];
    assert_eq!(newest["title"], "Bread, daily");
    assert_eq!(newest["pages"], json!([{"path": "index.html", "title": "Home"}, {"path": "menu/index.html", "title": "Menu"}]));
    assert_eq!(newest["elements"][0], json!({"id": "hero", "kind": "section", "label": "Hero", "pages": ["index.html"]}));
    assert_eq!(newest["published_url"], json!(null));
    assert!(newest["created_at"].is_string());
    assert_eq!(newest["preview_token"].as_str().unwrap().len(), design_artifact::PREVIEW_TOKEN_CHARS);

    assert_eq!(app.get(&uri, &bob).await.0, StatusCode::NOT_FOUND, "another person's chat isn't theirs to look at");
    assert_eq!(app.call("GET", &uri, None, None).await.0, StatusCode::UNAUTHORIZED);
    design_on(&pool, admin.id, false).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_versions_files_are_served_by_name_into_a_sandbox_with_no_sign_in(pool: PgPool) {
    let app = App::new(pool.clone());
    let ada = app.person("ada@acme.test").await;
    let session_id = chat_of(&pool, &ada).await;
    save(&pool, session_id, &ada, &preview("editorial", "Fresh at six")).await;
    let mut conn = pool.acquire().await.unwrap();
    let token = design_artifact::list(&mut conn, session_id).await.unwrap()[0].preview_token.clone();
    drop(conn);

    for index in [format!("/api/design/preview/{token}"), format!("/api/design/preview/{token}/"), format!("/api/design/preview/{token}/index.html")] {
        let (status, headers, body) = fetch(&app, &index).await;
        assert_eq!(status, StatusCode::OK, "{index}");
        assert_eq!(headers["content-type"], "text/html; charset=utf-8");
        let policy = headers["content-security-policy"].to_str().unwrap();
        assert!(policy.starts_with("sandbox allow-scripts;"), "its own origin, apart from Nomi's: {policy}");
        assert!(!policy.contains("allow-same-origin"));
        assert!(policy.contains("connect-src 'none'") && policy.contains("form-action 'none'"));
        assert_eq!(headers["referrer-policy"], "no-referrer", "the name in the address isn't passed on");
        assert_eq!(headers["x-content-type-options"], "nosniff");
        assert!(headers.get("set-cookie").is_none());
        assert!(body.contains("Fresh at six"));
        assert!(body.contains("nomi-design-preview") && body.contains("\"index.html\""), "the page can tell the Studio what was pointed at");
    }

    let (status, headers, body) = fetch(&app, &format!("/api/design/preview/{token}/styles.css")).await;
    assert_eq!((status, headers["content-type"].to_str().unwrap()), (StatusCode::OK, "text/css; charset=utf-8"));
    assert_eq!(body, "h1 { margin: 0 }", "only pages get the Studio's script");
    assert!(headers["content-security-policy"].to_str().unwrap().starts_with("sandbox"));

    let (status, _, body) = fetch(&app, &format!("/api/design/preview/{token}/menu/")).await;
    assert!(status == StatusCode::OK && body.contains("menu-list") && body.contains("\"menu/index.html\""), "a folder opens its index page");

    for missing in [format!("/api/design/preview/{token}/nope.html"), format!("/api/design/preview/{}/index.html", "0".repeat(design_artifact::PREVIEW_TOKEN_CHARS)), "/api/design/preview/short/index.html".to_string()] {
        assert_eq!(fetch(&app, &missing).await.0, StatusCode::NOT_FOUND, "{missing}");
    }
}
