//! A crew member's skills (Settings › Crew): typed, uploaded, imported from a link, switched on or
//! off, edited and removed, for this person only.

mod support;

use std::io::Write;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{App, Person};

const MEMBER: &str = "chitchat";

fn skill(name: &str) -> String {
    format!("---\nname: {name}\ndescription: Use when planning meals for the week.\n---\n# {name}\nBody")
}

fn skills_uri(member: &str) -> String {
    format!("/api/agents/{member}/skills")
}

async fn add(app: &App, who: &Person, name: &str) -> Value {
    let (status, body) = app.send("POST", &skills_uri(MEMBER), who, json!({ "text": skill(name) })).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body
}

async fn upload(app: &App, who: &Person, filename: &str, bytes: Vec<u8>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri(format!("{}/upload?filename={filename}", skills_uri(MEMBER)))
        .header("authorization", format!("Bearer {}", who.token))
        .header("content-type", "application/octet-stream")
        .body(Body::from(bytes))
        .unwrap();
    let (status, _, body) = app.raw(request).await;
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        out.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        out.write_all(bytes).unwrap();
    }
    out.finish().unwrap().into_inner()
}

#[sqlx::test(migrations = "../../migrations")]
async fn typed_create_list_get_update_delete(pool: PgPool) {
    let app = App::new(pool);
    let me = app.person("skills-crud@nomi.test").await;
    let made = app
        .send("POST", &skills_uri(MEMBER), &me, json!({ "text": skill("menu"), "references": [{ "name": "pantry.md", "text": "rice" }] }))
        .await;
    assert_eq!(made.0, StatusCode::CREATED, "{}", made.1);
    assert_eq!(made.1["name"], "menu");
    assert_eq!(made.1["origin"], "typed");
    assert_eq!(made.1["reference_names"], json!(["pantry"]));
    let id = made.1["id"].as_str().unwrap().to_string();

    let (status, listed) = app.get(&skills_uri(MEMBER), &me).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["builtin"], json!([]), "chitchat has no built-in skills");
    assert_eq!(listed["mine"][0]["name"], "menu");
    assert!(listed["mine"][0].get("text").is_none(), "the list leaves the text out");

    let (status, one) = app.get(&format!("{}/{id}", skills_uri(MEMBER)), &me).await;
    assert_eq!(status, StatusCode::OK);
    assert!(one["text"].as_str().unwrap().contains("# menu"));
    assert_eq!(one["references"], json!([{ "name": "pantry", "text": "rice" }]));

    let (status, updated) = app.send("PUT", &format!("{}/{id}", skills_uri(MEMBER)), &me, json!({ "text": skill("weekly-menu") })).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["name"], "weekly-menu");
    assert_eq!(updated["reference_names"], json!(["pantry"]), "references stay when only the text changes");

    let (status, _) = app.call("DELETE", &format!("{}/{id}", skills_uri(MEMBER)), Some(&me.token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, listed) = app.get(&skills_uri(MEMBER), &me).await;
    assert_eq!(listed["mine"], json!([]));
}

async fn post(app: &App, who: &Person, text: String) -> (StatusCode, Value) {
    app.send("POST", &skills_uri(MEMBER), who, json!({ "text": text })).await
}

async fn import(app: &App, who: &Person, url: &str) -> (StatusCode, Value) {
    app.send("POST", &format!("{}/import", skills_uri(MEMBER)), who, json!({ "url": url })).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_error_code(pool: PgPool) {
    let app = App::new(pool.clone());
    let me = app.person("skills-errors@nomi.test").await;

    let (status, body) = post(&app, &me, skill("Bad Name")).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("invalid_skill")), "{body}");
    assert!(body["detail"].as_str().unwrap().contains("name"));

    let (status, body) = post(&app, &me, format!("{}{}", skill("big"), "x".repeat(16_000))).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("too_large")));

    add(&app, &me, "menu").await;
    let (status, body) = post(&app, &me, skill("menu")).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("name_taken")));

    for i in 1..nomi_skills::MAX_SKILLS_PER_MEMBER {
        add(&app, &me, &format!("s{i}")).await;
    }
    let (status, body) = post(&app, &me, skill("one-more")).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("too_many_skills")));
    sqlx::query("DELETE FROM user_agent_skills WHERE name LIKE 's%'").execute(&pool).await.unwrap();

    let (status, body) = upload(&app, &me, "menu.pdf", b"%PDF".to_vec()).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("unsupported_file")));
    let (status, body) = upload(&app, &me, "menu.zip", b"not a zip".to_vec()).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("not_a_skill")));

    let (status, body) = import(&app, &me, "https://localhost/menu.md").await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("private_address")), "{body}");
    let (status, body) = import(&app, &me, "https://nomi-skill-test.invalid/menu.md").await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("fetch_failed")), "{body}");
    let (status, body) = import(&app, &me, "https://example.com/page").await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("not_a_skill")));

    let (_, listed) = app.get(&skills_uri(MEMBER), &me).await;
    let id = listed["mine"][0]["id"].as_str().unwrap().to_string();
    let (status, body) = app.call("POST", &format!("{}/{id}/refresh", skills_uri(MEMBER)), Some(&me.token), None).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("not_a_link_skill")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn upload_md_and_zip(pool: PgPool) {
    let app = App::new(pool);
    let me = app.person("skills-upload@nomi.test").await;
    let (status, body) = upload(&app, &me, "menu.md", skill("menu").into_bytes()).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["origin"], "upload");
    assert_eq!(body["skipped"], json!([]));

    let zipped = zip_of(&[("pantry/SKILL.md", skill("pantry").as_bytes()), ("pantry/references/staples.md", b"rice"), ("pantry/run.py", b"x")]);
    let (status, body) = upload(&app, &me, "pantry.zip", zipped).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["reference_names"], json!(["staples"]));
    assert_eq!(body["skipped"], json!(["pantry/run.py"]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn refresh_goes_back_to_the_link(pool: PgPool) {
    let app = App::new(pool.clone());
    let me = app.person("skills-refresh@nomi.test").await;
    // A link skill whose source now points at a private address: the refresh fetches it again,
    // through the same guard.
    let id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO user_agent_skills (user_id, agent_type, name, description, text, origin, source_url) \
         VALUES ($1, $2, 'menu', 'Use when planning meals for the week.', $3, 'link', 'https://localhost/menu.md') RETURNING id",
    )
    .bind(me.id)
    .bind(MEMBER)
    .bind(skill("menu"))
    .fetch_one(&pool)
    .await
    .unwrap();
    let (status, body) = app.call("POST", &format!("{}/{id}/refresh", skills_uri(MEMBER)), Some(&me.token), None).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("private_address")), "{body}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_member_and_other_users_skill_404(pool: PgPool) {
    let app = App::new(pool);
    let me = app.person("skills-owner@nomi.test").await;
    let other = app.person("skills-other@nomi.test").await;
    let (status, _) = app.get(&skills_uri("no-such-member"), &me).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = app.send("POST", &skills_uri("no-such-member"), &me, json!({ "text": skill("menu") })).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let made = add(&app, &me, "menu").await;
    let uri = format!("{}/{}", skills_uri(MEMBER), made["id"].as_str().unwrap());
    assert_eq!(app.get(&uri, &other).await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.send("PUT", &uri, &other, json!({ "enabled": false })).await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.call("DELETE", &uri, Some(&other.token), None).await.0, StatusCode::NOT_FOUND);
    let (_, listed) = app.get(&skills_uri(MEMBER), &other).await;
    assert_eq!(listed["mine"], json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn enabled_toggle_hides_from_index(pool: PgPool) {
    let app = App::new(pool.clone());
    let me = app.person("skills-toggle@nomi.test").await;
    let made = add(&app, &me, "menu").await;
    let uri = format!("{}/{}", skills_uri(MEMBER), made["id"].as_str().unwrap());
    let (status, body) = app.send("PUT", &uri, &me, json!({ "enabled": false })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["enabled"], false);
    let mut conn = pool.acquire().await.unwrap();
    assert!(nomi_tools_skills::store::index(&mut conn, me.id, MEMBER).await.unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_own_member_deletes_its_skills(pool: PgPool) {
    let app = App::new(pool.clone());
    let me = app.person("skills-member@nomi.test").await;
    let (status, made) = app.send("POST", "/api/agents", &me, json!({ "name": "Penny", "role": "Keeps an eye on the household budget." })).await;
    assert_eq!(status, StatusCode::CREATED, "{made}");
    let member = made["agent_type"].as_str().unwrap().to_string();
    let (status, body) = app.send("POST", &skills_uri(&member), &me, json!({ "text": skill("budget") })).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let (status, _) = app.call("DELETE", &format!("/api/agents/{member}"), Some(&me.token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM user_agent_skills WHERE user_id = $1").bind(me.id).fetch_one(&pool).await.unwrap();
    assert_eq!(left, 0);
}
