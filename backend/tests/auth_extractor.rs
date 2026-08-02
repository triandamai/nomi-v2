mod support;

use axum::{body::Body, http::{Request, StatusCode}, routing::get, Router};
use http_body_util::BodyExt;
use nomi_orchestrator::app::AppState;
use nomi_orchestrator::auth::{claims::Claims, extractor::AuthClaims};
use nomi_orchestrator::llm::{LlmResponse, StopReason};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_router(pool: PgPool) -> Router {
    let state = AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        provider: Arc::new(tokio::sync::RwLock::new(Arc::new(FakeLlmProvider::success(LlmResponse {
            content: vec![],
            stop_reason: StopReason::EndTurn,
            input_tokens: 0,
            output_tokens: 0,
        })) as Arc<dyn nomi_orchestrator::llm::LlmProvider>)),
        embedding_provider: Arc::new(tokio::sync::RwLock::new(
            Arc::new(FakeEmbeddingProvider::success(dummy_embedding())) as Arc<dyn nomi_orchestrator::embedding::EmbeddingProvider>,
        )),
        http_client: reqwest::Client::new(),
        settings_key: support::TEST_SETTINGS_KEY,
    };
    Router::new()
        .route("/whoami", get(|AuthClaims(claims): AuthClaims| async move {
            axum::Json(claims)
        }))
        .with_state(state)
}

#[sqlx::test]
async fn valid_token_reaches_the_handler(pool: PgPool) {
    let claims = Claims::new(Uuid::new_v4(), Uuid::new_v4(), vec![], 1800);
    let token = claims.encode(SECRET).unwrap();

    let response = test_router(pool)
        .oneshot(
            Request::builder()
                .uri("/whoami")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let decoded: Claims = serde_json::from_slice(&body).unwrap();
    assert_eq!(decoded.sub, claims.sub);
}

#[sqlx::test]
async fn missing_token_is_rejected(pool: PgPool) {
    let response = test_router(pool)
        .oneshot(Request::builder().uri("/whoami").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn malformed_token_is_rejected(pool: PgPool) {
    let response = test_router(pool)
        .oneshot(
            Request::builder()
                .uri("/whoami")
                .header("authorization", "Bearer not-a-real-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn expired_token_is_rejected(pool: PgPool) {
    let mut claims = Claims::new(Uuid::new_v4(), Uuid::new_v4(), vec![], 1800);
    // -3600 (not -60): jsonwebtoken's default 60-second leeway makes -60 borderline/unreliable.
    claims.exp = chrono::Utc::now().timestamp() - 3600;
    let token = claims.encode(SECRET).unwrap();

    let response = test_router(pool)
        .oneshot(
            Request::builder()
                .uri("/whoami")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
