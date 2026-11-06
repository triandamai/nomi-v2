//! Shared by the route tests added for coverage: the app with test settings, requests as a
//! signed-in person, and the people and projects tests start from.
#![allow(dead_code)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

pub const SECRET: &str = "test-secret-do-not-use-in-prod";

pub fn state(pool: PgPool) -> AppState {
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

pub struct App {
    pub pool: PgPool,
    pub router: axum::Router,
}

pub struct Person {
    pub id: Uuid,
    pub token: String,
    pub email: String,
}

impl App {
    pub fn new(pool: PgPool) -> Self {
        let router = build_router(state(pool.clone()));
        App { pool, router }
    }

    pub fn with_state(pool: PgPool, state: AppState) -> Self {
        App { pool, router: build_router(state) }
    }

    /// Sends a request; the body comes back as JSON, or as a JSON string when it isn't JSON.
    pub async fn call(&self, method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        let body = match body {
            Some(b) => {
                builder = builder.header("content-type", "application/json");
                Body::from(b.to_string())
            }
            None => Body::empty(),
        };
        let response = self.router.clone().oneshot(builder.body(body).unwrap()).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };
        (status, value)
    }

    pub async fn raw(&self, request: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes().to_vec();
        (status, headers, bytes)
    }

    pub async fn get(&self, uri: &str, who: &Person) -> (StatusCode, Value) {
        self.call("GET", uri, Some(&who.token), None).await
    }

    pub async fn send(&self, method: &str, uri: &str, who: &Person, body: Value) -> (StatusCode, Value) {
        self.call(method, uri, Some(&who.token), Some(body)).await
    }

    /// Registers (with an organization of their own) and signs in.
    pub async fn person(&self, email: &str) -> Person {
        let (status, body) = self
            .call("POST", "/api/auth/register", None, Some(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })))
            .await;
        assert!(status.is_success(), "register {email}: {status} {body}");
        self.login(email).await
    }

    pub async fn login(&self, email: &str) -> Person {
        let (status, body) = self.call("POST", "/api/auth/login", None, Some(json!({ "email": email, "password": "correct-password" }))).await;
        assert!(status.is_success(), "login {email}: {status} {body}");
        let id = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(&self.pool).await.unwrap();
        Person { id, token: body["access_token"].as_str().unwrap().to_string(), email: email.to_string() }
    }

    /// A platform admin (every admin area at manage), signed in again so the token says so.
    pub async fn admin(&self, email: &str) -> Person {
        self.person(email).await;
        sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
            .bind(email)
            .execute(&self.pool)
            .await
            .unwrap();
        self.login(email).await
    }

    /// A project of `owner`'s with its own chat, in the owner's organization.
    pub async fn project(&self, owner: &Person, name: &str) -> (Uuid, Uuid) {
        let org: Uuid = sqlx::query_scalar("SELECT org_id FROM memberships WHERE user_id = $1 LIMIT 1").bind(owner.id).fetch_one(&self.pool).await.unwrap();
        let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', $2) RETURNING id")
            .bind(org)
            .bind(Uuid::new_v4().to_string())
            .fetch_one(&self.pool)
            .await
            .unwrap();
        let project: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, org_id) VALUES ($1, $2, $3, $4) RETURNING id")
            .bind(owner.id)
            .bind(session)
            .bind(name)
            .bind(org)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        (project, session)
    }
}
