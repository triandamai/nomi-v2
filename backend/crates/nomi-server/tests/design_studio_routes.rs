//! The Design Studio's routes: a chat's previews listed for the people who can open the chat
//! (where Design is rolled out), and a version's files served to the Studio's sandboxed frame
//! by an unguessable name, with no sign-in and nothing of Nomi's within reach.

mod support;


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
    nomi_settings::config::set_agent_on(pool, nomi_agent_core::crew::DESIGN_AGENT_TYPE, on, admin).await.unwrap();
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
    assert_eq!(body["directions"], json!(null), "directions is null until Sora proposes a set");
    let previews = body["previews"].as_array().unwrap();
    assert_eq!(previews.iter().map(|p| (p["version"].as_i64().unwrap(), p["direction_id"].as_str().unwrap())).collect::<Vec<_>>(), [(1, "editorial"), (2, "swiss")]);
    let newest = &previews[1];
    assert_eq!(newest["title"], "Bread, daily");
    assert_eq!(newest["kind"], "design", "the Studio is told what kind of thing each version is");
    assert_eq!(newest["pages"], json!([{"path": "index.html", "title": "Home"}, {"path": "menu/index.html", "title": "Menu"}]));
    assert_eq!(newest["elements"][0], json!({"id": "hero", "kind": "section", "label": "Hero", "pages": ["index.html"]}));
    assert_eq!(newest["published_url"], json!(null));
    assert!(newest["created_at"].is_string());
    assert_eq!(newest["preview_token"].as_str().unwrap().len(), design_artifact::PREVIEW_TOKEN_CHARS);

    let set = nomi_agent_core::design_directions::DesignDirections::parse(&json!({
        "directions": [
            {
                "id": "editorial",
                "name": "Editorial",
                "summary": "Warm paper tones and serif headings.",
                "rationale": "Fits the craft bakery story.",
                "trade_offs": ["Needs strong photography"],
                "palette": [{"name": "Paper", "hex": "#F6EFE4"}, {"name": "Accent", "hex": "#B5532A"}],
                "typography": {"heading": "Fraunces", "body": "Inter"},
                "layout": "Hero, daily bakes, pre-order",
                "recommended": true
            },
            {
                "id": "swiss",
                "name": "Swiss utility",
                "summary": "Crisp grid and monospace prices.",
                "rationale": "Fast menu scanning.",
                "trade_offs": ["Less warmth"],
                "palette": [{"name": "Surface", "hex": "#FFFFFF"}, {"name": "Ink", "hex": "#111111"}],
                "typography": {"heading": "Space Grotesk", "body": "IBM Plex Sans"},
                "layout": "Two-column menu table"
            }
        ]
    }))
    .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    nomi_agent_core::design_directions::save(&mut conn, session_id, Uuid::new_v4(), ada.id, Some(1), &set).await.unwrap();
    drop(conn);

    let (status, with_directions) = app.get(&uri, &ada).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(with_directions["directions"]["version"], 1);
    assert_eq!(with_directions["directions"]["brief_version"], 1);
    assert_eq!(with_directions["directions"]["directions"].as_array().unwrap().len(), 2);
    assert_eq!(with_directions["directions"]["directions"][0]["id"], "editorial");

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
        assert!(policy.contains("frame-ancestors 'self'"), "another site can't put it in a frame");
        assert_eq!(headers["x-robots-tag"], "noindex, nofollow");
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

#[sqlx::test(migrations = "../../migrations")]
async fn every_preview_serves_the_deck_runtime_beside_its_own_files(pool: PgPool) {
    let app = App::new(pool.clone());
    let ada = app.person("ada@acme.test").await;
    let session_id = chat_of(&pool, &ada).await;
    save(&pool, session_id, &ada, &preview("editorial", "Fresh at six")).await;
    let mut conn = pool.acquire().await.unwrap();
    let token = design_artifact::list(&mut conn, session_id).await.unwrap()[0].preview_token.clone();
    drop(conn);

    let (status, headers, body) = fetch(&app, &format!("/api/design/preview/{token}/_runtime/html-ppt/runtime.js")).await;
    assert_eq!((status, headers["content-type"].to_str().unwrap()), (StatusCode::OK, "text/javascript; charset=utf-8"));
    assert!(body.contains("BroadcastChannel"), "the deck runtime itself");
    assert!(headers["content-security-policy"].to_str().unwrap().starts_with("sandbox"), "fenced in like the preview's own files");
    let (status, headers, _) = fetch(&app, &format!("/api/design/preview/{token}/_runtime/html-ppt/themes/academic-paper.css")).await;
    assert_eq!((status, headers["content-type"].to_str().unwrap()), (StatusCode::OK, "text/css; charset=utf-8"));
    let (status, _, licence) = fetch(&app, &format!("/api/design/preview/{token}/_runtime/html-ppt/LICENSE")).await;
    assert!(status == StatusCode::OK && licence.contains("MIT License"));
    for missing in [format!("/api/design/preview/{token}/_runtime/html-ppt/nope.js"), format!("/api/design/preview/{}/_runtime/html-ppt/runtime.js", "0".repeat(design_artifact::PREVIEW_TOKEN_CHARS))] {
        assert_eq!(fetch(&app, &missing).await.0, StatusCode::NOT_FOUND, "{missing}: only a real version's address serves it");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn feedback_from_the_studio_is_kept_for_the_chat_and_listed_with_its_previews(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let admin = app.admin("admin@acme.test").await;
    let ada = app.person("ada@acme.test").await;
    let bob = app.person("bob@acme.test").await;
    let session_id = chat_of(&pool, &ada).await;
    save(&pool, session_id, &ada, &preview("editorial", "Fresh at six")).await;
    let uri = format!("/api/sessions/{session_id}/design/feedback");
    let on_hero = json!({"version": 1, "level": "element", "page": "index.html", "element_id": "hero", "body": "Make it shorter", "pin": {"x": 0.5, "y": 0.25}});

    design_on(&pool, admin.id, false).await;
    assert_eq!(app.send("POST", &uri, &ada, on_hero.clone()).await.0, StatusCode::NOT_FOUND, "no feedback where Design isn't rolled out");

    design_on(&pool, admin.id, true).await;
    let (status, first) = app.send("POST", &uri, &ada, on_hero.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{first}");
    assert_eq!((first["number"].as_i64(), first["level"].as_str(), first["status"].as_str()), (Some(1), Some("section"), Some("open")), "the manifest says the hero is a section");
    assert_eq!(first["pin"], json!({"x": 0.5, "y": 0.25}));
    let (status, second) = app.send("POST", &uri, &ada, json!({"version": 1, "level": "global", "body": "Warmer overall"})).await;
    assert_eq!((status, second["number"].as_i64()), (StatusCode::CREATED, Some(2)));

    let (status, refused) = app.send("POST", &uri, &ada, json!({"version": 1, "level": "page", "page": "about.html", "body": "x"})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(refused, json!("this version has no page \"about.html\""));
    assert_eq!(app.send("POST", &uri, &ada, json!({"version": 9, "level": "global", "body": "x"})).await.0, StatusCode::NOT_FOUND, "feedback is about a version there is");
    assert_eq!(app.send("POST", &uri, &ada, json!({"level": "global", "body": "x"})).await.0, StatusCode::UNPROCESSABLE_ENTITY, "it names its version");
    assert_eq!(app.send("POST", &uri, &bob, on_hero.clone()).await.0, StatusCode::NOT_FOUND, "another person's chat isn't theirs to comment on");
    assert_eq!(app.call("POST", &uri, None, Some(on_hero)).await.0, StatusCode::UNAUTHORIZED);

    let (_, studio) = app.get(&format!("/api/sessions/{session_id}/design"), &ada).await;
    let kept = studio["feedback"].as_array().unwrap();
    assert_eq!(kept.iter().map(|f| (f["number"].as_i64().unwrap(), f["body"].as_str().unwrap())).collect::<Vec<_>>(), [(1, "Make it shorter"), (2, "Warmer overall")]);
    assert_eq!((kept[0]["version"].as_i64(), kept[0]["element_id"].as_str(), kept[0]["page"].as_str()), (Some(1), Some("hero"), Some("index.html")));
    design_on(&pool, admin.id, false).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_chat_can_only_have_so_much_feedback_waiting(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let admin = app.admin("admin@acme.test").await;
    let ada = app.person("ada@acme.test").await;
    let session_id = chat_of(&pool, &ada).await;
    save(&pool, session_id, &ada, &preview("editorial", "Fresh at six")).await;
    sqlx::query("INSERT INTO design_feedback (session_id, user_id, number, artifact_version, level, body) SELECT $1, $2, n, 1, 'global', 'x' FROM generate_series(1, $3) n")
        .bind(session_id)
        .bind(ada.id)
        .bind(nomi_agent_core::design_feedback::MAX_OPEN_PER_CHAT as i32)
        .execute(&pool)
        .await
        .unwrap();
    design_on(&pool, admin.id, true).await;
    let (status, refused) = app.send("POST", &format!("/api/sessions/{session_id}/design/feedback"), &ada, json!({"version": 1, "level": "global", "body": "One more"})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(refused.as_str().unwrap().contains("waiting"), "{refused}");
    design_on(&pool, admin.id, false).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_person_approves_a_version_and_the_newest_approval_stands(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let admin = app.admin("admin@acme.test").await;
    let ada = app.person("ada@acme.test").await;
    let bob = app.person("bob@acme.test").await;
    let session_id = chat_of(&pool, &ada).await;
    save(&pool, session_id, &ada, &preview("editorial", "Fresh at six")).await;
    save(&pool, session_id, &ada, &preview("editorial", "Fresh at six, daily")).await;
    let uri = format!("/api/sessions/{session_id}/design/approval");
    let studio = format!("/api/sessions/{session_id}/design");

    design_on(&pool, admin.id, false).await;
    assert_eq!(app.send("POST", &uri, &ada, json!({"version": 1})).await.0, StatusCode::NOT_FOUND);

    design_on(&pool, admin.id, true).await;
    assert_eq!(app.get(&studio, &ada).await.1["approval"], json!(null), "nothing is approved until the person says so");
    assert_eq!(app.send("POST", &uri, &ada, json!({"version": 5})).await.0, StatusCode::NOT_FOUND, "only a version there is");
    assert_eq!(app.send("POST", &uri, &ada, json!({})).await.0, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(app.send("POST", &uri, &bob, json!({"version": 1})).await.0, StatusCode::NOT_FOUND, "another person's chat isn't theirs to approve");
    assert_eq!(app.call("POST", &uri, None, Some(json!({"version": 1}))).await.0, StatusCode::UNAUTHORIZED);

    let (status, first) = app.send("POST", &uri, &ada, json!({"version": 1, "note": "Go with this"})).await;
    assert_eq!(status, StatusCode::CREATED, "{first}");
    assert_eq!((first["version"].as_i64(), first["note"].as_str()), (Some(1), Some("Go with this")));
    assert_eq!(app.get(&studio, &ada).await.1["approval"]["version"], 1);
    // Approving is the go-ahead: Sora is handed the handoff without a word in the chat.
    let tasks: Vec<(String, String)> = sqlx::query_as("SELECT target_agent_type, task FROM agent_delegations WHERE session_id = $1").bind(session_id).fetch_all(&pool).await.unwrap();
    assert_eq!(tasks.len(), 1, "{tasks:?}");
    assert_eq!(tasks[0].0, "design");
    assert!(tasks[0].1.contains("approved version 1") && tasks[0].1.contains("create_handoff"), "{}", tasks[0].1);

    assert_eq!(app.send("POST", &uri, &ada, json!({"version": 2})).await.0, StatusCode::CREATED);
    let (_, body) = app.get(&studio, &ada).await;
    assert_eq!((body["approval"]["version"].as_i64(), body["approval"]["note"].is_null()), (Some(2), true), "the newest approval stands");
    let mut conn = pool.acquire().await.unwrap();
    let history = nomi_agent_core::design_approval::history(&mut conn, session_id).await.unwrap();
    assert_eq!(history.iter().map(|a| a.version).collect::<Vec<_>>(), [1, 2], "the earlier approval is still on record");
    assert_eq!(history[0].user_id, ada.id);
    design_on(&pool, admin.id, false).await;
}
