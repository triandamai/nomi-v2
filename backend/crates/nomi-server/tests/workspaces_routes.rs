//! Workspaces: a person's pending invites (list, decline), which one is personal, each card's
//! faces and activity, where Nomi opens, a workspace's own page (who made it, each member's work,
//! resending, invite links, deleting), and signing up from a project invite.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
        s3: None,
        project_storage: nomi_test_support::test_project_storage(),
        tool_catalog: std::sync::Arc::new(nomi_agent_core::ToolCatalog::empty()),
        email_codes: nomi_server::sign_in_codes::EmailCodes::Off,
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
    }
}

async fn send(
    router: &axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(bearer) = bearer {
        request = request.header("authorization", format!("Bearer {bearer}"));
    }
    let request = request
        .body(Body::from(if body.is_null() {
            String::new()
        } else {
            body.to_string()
        }))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn call(
    router: &axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: &str,
) -> (StatusCode, Value) {
    send(router, method, uri, body, Some(bearer)).await
}

async fn log_in(router: &axum::Router, email: &str) -> String {
    let (status, body) = send(
        router,
        "POST",
        "/api/auth/login",
        json!({"email": email, "password": "correct-password"}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["access_token"].as_str().unwrap().to_string()
}

async fn sign_up(router: &axum::Router, email: &str) -> String {
    let org = json!({"mode": "create", "name": "Home"});
    let (status, _) = send(
        router,
        "POST",
        "/api/auth/register",
        json!({"email": email, "password": "correct-password", "org": org}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    log_in(router, email).await
}

async fn active_org(router: &axum::Router, token: &str) -> String {
    let (_, claims) = call(router, "GET", "/api/whoami", Value::Null, token).await;
    claims["active_org_id"].as_str().unwrap().to_string()
}

async fn new_org(router: &axum::Router, token: &str, name: &str) -> String {
    let (status, org) = call(router, "POST", "/api/orgs", json!({"name": name}), token).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(org["is_personal"], false);
    assert_eq!(org["member_avatars"].as_array().unwrap().len(), 1);
    org["id"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn pending_invites_are_listed_declined_and_joined(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let bob = sign_up(&router, "bob@example.com").await;
    let studio = new_org(&router, &ana, "Studio").await;
    let kopi = new_org(&router, &ana, "Kopi Kita").await;
    let (_, created) = call(
        &router,
        "POST",
        "/api/projects",
        json!({"org_id": kopi}),
        &ana,
    )
    .await;
    let project_id = created["project_id"].as_str().unwrap().to_string();

    let (_, studio_invite) = call(
        &router,
        "POST",
        &format!("/api/orgs/{studio}/invites"),
        json!({"email": "bob@example.com"}),
        &ana,
    )
    .await;
    let (_, project_invite) = call(
        &router,
        "POST",
        &format!("/api/projects/{project_id}/invites"),
        json!({"email": "bob@example.com", "role": "build"}),
        &ana,
    )
    .await;
    let project_code = project_invite["path"]
        .as_str()
        .unwrap()
        .trim_start_matches("/invite/")
        .to_string();

    // Bob sees both, newest first, with who asked and what's in each.
    let (status, listed) = call(&router, "GET", "/api/invites", Value::Null, &bob).await;
    assert_eq!(status, StatusCode::OK);
    let invites = listed["invites"].as_array().unwrap();
    assert_eq!(invites.len(), 2);
    assert_eq!(invites[0]["code"], project_code.as_str());
    assert_eq!(invites[0]["org_name"], "Kopi Kita");
    assert_eq!(invites[0]["inviter"], "ana");
    assert_eq!(invites[0]["projects"], 1);
    assert_eq!(invites[0]["project"]["id"], project_id.as_str());
    assert_eq!(invites[1]["org_id"], studio.as_str());
    assert_eq!(invites[1]["project"], Value::Null);
    // Ana, who sent them, has none of her own.
    let (_, none) = call(&router, "GET", "/api/invites", Value::Null, &ana).await;
    assert_eq!(none["invites"].as_array().unwrap().len(), 0);

    // Someone else can't decline Bob's invite; Bob can, and it leaves both lists.
    let carol = sign_up(&router, "carol@example.com").await;
    let studio_code = studio_invite["code"].as_str().unwrap();
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/invites/{studio_code}/decline"),
            Value::Null,
            &carol
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/invites/{studio_code}/decline"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/invites/{studio_code}/decline"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    let (_, info) = call(
        &router,
        "GET",
        &format!("/api/invites/{studio_code}"),
        Value::Null,
        &bob,
    )
    .await;
    assert_eq!(info["state"], "declined");
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/invites/{studio_code}/accept"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::GONE
    );
    let (_, studio_page) = call(
        &router,
        "GET",
        &format!("/api/orgs/{studio}"),
        Value::Null,
        &ana,
    )
    .await;
    assert_eq!(studio_page["invites"].as_array().unwrap().len(), 0);
    let (_, listed) = call(&router, "GET", "/api/invites", Value::Null, &bob).await;
    assert_eq!(listed["invites"].as_array().unwrap().len(), 1);

    // Resending brings a declined one back.
    let (status, resent) = call(
        &router,
        "POST",
        &format!("/api/orgs/{studio}/invites/{studio_code}/resend"),
        Value::Null,
        &ana,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resent["email"], "bob@example.com");
    let (_, listed) = call(&router, "GET", "/api/invites", Value::Null, &bob).await;
    assert_eq!(listed["invites"].as_array().unwrap().len(), 2);
    // Only owners and admins resend, and only invites that exist.
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/orgs/{studio}/invites/{studio_code}/resend"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/orgs/{studio}/invites/nope/resend"),
            Value::Null,
            &ana
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );

    // Joining takes it off his list, and there's nothing left to decline.
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/invites/{project_code}/accept"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::OK
    );
    let (_, listed) = call(&router, "GET", "/api/invites", Value::Null, &bob).await;
    assert_eq!(listed["invites"].as_array().unwrap().len(), 1);
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/invites/{project_code}/decline"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn cards_say_personal_faces_and_activity(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let studio = new_org(&router, &ana, "Studio").await;
    // Ana's sign-up workspace is named, so not personal; mark one as a channel sign-up's would be.
    let home = active_org(&router, &ana).await;
    sqlx::query("UPDATE organizations SET is_personal = true WHERE id = $1")
        .bind(Uuid::parse_str(&home).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    let (_, created) = call(
        &router,
        "POST",
        "/api/projects",
        json!({"org_id": studio}),
        &ana,
    )
    .await;
    let session_id = created["session_id"].as_str().unwrap();
    call(
        &router,
        "POST",
        &format!("/api/sessions/{session_id}/messages"),
        json!({"text": "A menu page"}),
        &ana,
    )
    .await;

    let (status, listed) = call(&router, "GET", "/api/orgs", Value::Null, &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["open_org_id"], Value::Null);
    let orgs = listed["orgs"].as_array().unwrap();
    let home_card = orgs.iter().find(|o| o["id"] == home.as_str()).unwrap();
    assert_eq!(home_card["is_personal"], true);
    assert_eq!(home_card["last_activity"], Value::Null);
    let studio_card = orgs.iter().find(|o| o["id"] == studio.as_str()).unwrap();
    assert_eq!(studio_card["is_personal"], false);
    assert_eq!(studio_card["member_avatars"][0]["name"], "ana");
    assert_eq!(studio_card["last_activity"]["kind"], "worked_on");
    assert_eq!(studio_card["last_activity"]["by"], "ana");
    assert!(studio_card["last_activity"]["at"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn sign_in_opens_the_chosen_workspace(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let home = active_org(&router, &ana).await;
    let studio = new_org(&router, &ana, "Studio").await;

    // Last used (the default): she signs in where she was.
    assert_eq!(
        active_org(&router, &log_in(&router, "ana@example.com").await).await,
        home
    );

    assert_eq!(
        call(
            &router,
            "PUT",
            "/api/preferences/open-org",
            json!({"org_id": studio}),
            &ana
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    let (_, listed) = call(&router, "GET", "/api/orgs", Value::Null, &ana).await;
    assert_eq!(listed["open_org_id"], studio.as_str());
    let token = log_in(&router, "ana@example.com").await;
    assert_eq!(active_org(&router, &token).await, studio);

    // Switching still works within the session; the next sign-in opens Studio again.
    let (_, switched) = call(
        &router,
        "PUT",
        "/api/auth/active-org",
        json!({"org_id": home}),
        &token,
    )
    .await;
    assert_eq!(
        active_org(&router, switched["access_token"].as_str().unwrap()).await,
        home
    );
    assert_eq!(
        active_org(&router, &log_in(&router, "ana@example.com").await).await,
        studio
    );

    // Back to last used: she opens where she last switched to.
    assert_eq!(
        call(
            &router,
            "PUT",
            "/api/preferences/open-org",
            json!({"org_id": null}),
            &ana
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    let (_, switched) = call(
        &router,
        "PUT",
        "/api/auth/active-org",
        json!({"org_id": home}),
        &token,
    )
    .await;
    assert!(switched["access_token"].is_string());
    assert_eq!(
        active_org(&router, &log_in(&router, "ana@example.com").await).await,
        home
    );

    // Only a workspace she's in.
    let bob = sign_up(&router, "bob@example.com").await;
    let bobs = active_org(&router, &bob).await;
    assert_eq!(
        call(
            &router,
            "PUT",
            "/api/preferences/open-org",
            json!({"org_id": bobs}),
            &ana
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_workspace_page_shows_its_maker_members_work_and_links(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let bob = sign_up(&router, "bob@example.com").await;
    let studio = new_org(&router, &ana, "Studio").await;
    call(
        &router,
        "POST",
        "/api/projects",
        json!({"org_id": studio}),
        &ana,
    )
    .await;

    // A link anyone can join with, for a role; only managers make one.
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/orgs/{studio}/invite-links"),
            json!({}),
            &bob
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/orgs/{studio}/invite-links"),
            json!({"role": "owner"}),
            &ana
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, link) = call(
        &router,
        "POST",
        &format!("/api/orgs/{studio}/invite-links"),
        json!({}),
        &ana,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(link["link"], true);
    assert_eq!(link["email"], Value::Null);
    assert_eq!(link["role"], "member");
    let code = link["code"].as_str().unwrap().to_string();
    // A fresh one replaces it.
    let (_, link) = call(
        &router,
        "POST",
        &format!("/api/orgs/{studio}/invite-links"),
        json!({"role": "member"}),
        &ana,
    )
    .await;
    assert_eq!(
        call(
            &router,
            "GET",
            &format!("/api/invites/{code}"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let code = link["code"].as_str().unwrap().to_string();
    // Resending a link has nobody to send to.
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/orgs/{studio}/invites/{code}/resend"),
            Value::Null,
            &ana
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );

    // Two people join with the same link; it stays open, and a link can't be declined.
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/invites/{code}/decline"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/invites/{code}/accept"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::OK
    );
    let carol = sign_up(&router, "carol@example.com").await;
    assert_eq!(
        call(
            &router,
            "POST",
            &format!("/api/invites/{code}/accept"),
            Value::Null,
            &carol
        )
        .await
        .0,
        StatusCode::OK
    );
    // Someone new signs up with it too.
    let org = json!({"mode": "join", "invite_code": code});
    let (status, _) = send(
        &router,
        "POST",
        "/api/auth/register",
        json!({"email": "dan@example.com", "password": "correct-password", "org": org}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, page) = call(
        &router,
        "GET",
        &format!("/api/orgs/{studio}"),
        Value::Null,
        &ana,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page["created_by"]["name"], "ana");
    assert!(page["created_at"].is_string());
    assert_eq!(page["is_personal"], false);
    let members = page["members"].as_array().unwrap();
    assert_eq!(members.len(), 4);
    assert_eq!(members[0]["name"], "ana");
    assert_eq!(members[0]["projects"], 1);
    assert!(members[0]["last_active_at"].is_string());
    let bob_row = members.iter().find(|m| m["name"] == "bob").unwrap();
    assert_eq!(bob_row["projects"], 0);
    let invites = page["invites"].as_array().unwrap();
    assert_eq!(invites.len(), 1);
    assert_eq!(invites[0]["link"], true);
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_owners_delete_an_empty_workspace(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let bob = sign_up(&router, "bob@example.com").await;
    let home = active_org(&router, &ana).await;
    let studio = new_org(&router, &ana, "Studio").await;
    let (_, invite) = call(
        &router,
        "POST",
        &format!("/api/orgs/{studio}/invites"),
        json!({"email": "bob@example.com", "role": "admin"}),
        &ana,
    )
    .await;
    call(
        &router,
        "POST",
        &format!("/api/invites/{}/accept", invite["code"].as_str().unwrap()),
        Value::Null,
        &bob,
    )
    .await;
    let (_, open_invite) = call(
        &router,
        "POST",
        &format!("/api/orgs/{studio}/invites"),
        json!({"email": "erin@example.com"}),
        &ana,
    )
    .await;

    // Ana works in Studio: a chat of hers there, and Studio is where she opens.
    let (_, switched) = call(
        &router,
        "PUT",
        "/api/auth/active-org",
        json!({"org_id": studio}),
        &ana,
    )
    .await;
    let ana_in_studio = switched["access_token"].as_str().unwrap().to_string();
    let (status, chat) = call(&router, "POST", "/api/sessions", json!({}), &ana_in_studio).await;
    assert!(status.is_success(), "{chat}");
    let chat_id =
        Uuid::parse_str(chat["id"].as_str().or(chat["session_id"].as_str()).unwrap()).unwrap();
    call(
        &router,
        "PUT",
        "/api/preferences/open-org",
        json!({"org_id": studio}),
        &ana,
    )
    .await;

    // An admin can't; a workspace with projects can't go yet; nor can a personal one.
    assert_eq!(
        call(
            &router,
            "DELETE",
            &format!("/api/orgs/{studio}"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (_, created) = call(
        &router,
        "POST",
        "/api/projects",
        json!({"org_id": studio}),
        &ana,
    )
    .await;
    let project_id = Uuid::parse_str(created["project_id"].as_str().unwrap()).unwrap();
    assert_eq!(
        call(
            &router,
            "DELETE",
            &format!("/api/orgs/{studio}"),
            Value::Null,
            &ana
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    sqlx::query("UPDATE projects SET org_id = $2 WHERE id = $1")
        .bind(project_id)
        .bind(Uuid::parse_str(&home).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE organizations SET is_personal = true WHERE id = $1")
        .bind(Uuid::parse_str(&home).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        call(
            &router,
            "DELETE",
            &format!("/api/orgs/{home}"),
            Value::Null,
            &ana
        )
        .await
        .0,
        StatusCode::CONFLICT
    );

    assert_eq!(
        call(
            &router,
            "DELETE",
            &format!("/api/orgs/{studio}"),
            Value::Null,
            &ana
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    // It's gone for everyone, with its open invites; Ana's chat went home with her.
    assert_eq!(
        call(
            &router,
            "GET",
            &format!("/api/orgs/{studio}"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let (_, listed) = call(&router, "GET", "/api/orgs", Value::Null, &ana).await;
    assert!(listed["orgs"]
        .as_array()
        .unwrap()
        .iter()
        .all(|o| o["id"] != studio.as_str()));
    assert_eq!(listed["open_org_id"], Value::Null);
    let code = open_invite["code"].as_str().unwrap();
    assert_eq!(
        call(
            &router,
            "GET",
            &format!("/api/invites/{code}"),
            Value::Null,
            &bob
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let chat_org: Uuid = sqlx::query_scalar("SELECT org_id FROM sessions WHERE id = $1")
        .bind(chat_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(chat_org.to_string(), home);
    assert_eq!(
        active_org(&router, &log_in(&router, "ana@example.com").await).await,
        home
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_lone_members_only_workspace_stays(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let home = active_org(&router, &ana).await;
    assert_eq!(
        call(
            &router,
            "DELETE",
            &format!("/api/orgs/{home}"),
            Value::Null,
            &ana
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn signing_up_from_a_project_invite_puts_them_on_the_project(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = sign_up(&router, "ana@example.com").await;
    let studio = new_org(&router, &ana, "Studio").await;
    let (_, created) = call(
        &router,
        "POST",
        "/api/projects",
        json!({"org_id": studio}),
        &ana,
    )
    .await;
    let project_id = created["project_id"].as_str().unwrap().to_string();
    let session_id = Uuid::parse_str(created["session_id"].as_str().unwrap()).unwrap();
    let (_, invite) = call(
        &router,
        "POST",
        &format!("/api/projects/{project_id}/invites"),
        json!({"email": "bob@example.com", "role": "view"}),
        &ana,
    )
    .await;
    let code = invite["path"]
        .as_str()
        .unwrap()
        .trim_start_matches("/invite/")
        .to_string();

    let org = json!({"mode": "join", "invite_code": code});
    let (status, _) = send(
        &router,
        "POST",
        "/api/auth/register",
        json!({"email": "bob@example.com", "password": "correct-password", "org": org}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let bob = log_in(&router, "bob@example.com").await;
    assert_eq!(active_org(&router, &bob).await, studio);

    let role: String = sqlx::query_scalar("SELECT pm.role FROM project_members pm JOIN web_credentials w ON w.user_id = pm.user_id WHERE pm.project_id = $1 AND w.email = 'bob@example.com'")
        .bind(Uuid::parse_str(&project_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(role, "viewer");
    // The invite is used up, by him, and the project's chat says he joined.
    let used_by: Option<Uuid> =
        sqlx::query_scalar("SELECT used_by FROM org_invites WHERE code = $1")
            .bind(&code)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(used_by.is_some());
    let joined: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1 AND content_blocks::text LIKE '%member_joined%'")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(joined, 1);
}
