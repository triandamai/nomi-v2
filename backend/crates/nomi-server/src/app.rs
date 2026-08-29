use axum::{routing::{delete, get, post, put}, Router};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use nomi_auth::extractor::AuthClaims;
use crate::routes::admin_dashboard as admin_dashboard_routes;
use crate::routes::admin_users as admin_users_routes;
use crate::routes::auth as auth_routes;
use crate::routes::llm_models as llm_models_routes;
use crate::routes::personality as personality_routes;
use crate::routes::sessions as sessions_routes;
use crate::routes::settings as settings_routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
    pub mqtt_broker_host: String,
    pub mqtt_broker_port: u16,
}

impl nomi_auth::extractor::HasJwtSecret for AppState {
    fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/auth/register", post(auth_routes::register))
        .route("/api/auth/login", post(auth_routes::login_handler))
        .route("/api/auth/refresh", post(auth_routes::refresh_handler))
        .route("/api/auth/logout", post(auth_routes::logout_handler))
        .route(
            "/api/whoami",
            get(|AuthClaims(claims): AuthClaims| async move { axum::Json(claims) }),
        )
        .route("/api/orgs/:org_id/members/:user_id", delete(auth_routes::remove_member))
        .route(
            "/api/sessions",
            post(sessions_routes::create_session).get(sessions_routes::list_sessions),
        )
        .route(
            "/api/sessions/:id/messages",
            get(sessions_routes::list_messages).post(sessions_routes::send_message),
        )
        .route("/api/sessions/:id/ws", get(sessions_routes::session_stream))
        .route("/api/sessions/:id/agent-activity", get(sessions_routes::list_agent_activity))
        .route(
            "/api/admin/settings/llm/models",
            get(llm_models_routes::list_admin_models).post(llm_models_routes::create_admin_model),
        )
        .route(
            "/api/admin/settings/llm/models/:id",
            put(llm_models_routes::update_admin_model).delete(llm_models_routes::delete_admin_model),
        )
        .route(
            "/api/admin/settings/llm/models/:id/default",
            put(llm_models_routes::set_default_admin_model),
        )
        .route("/api/llm/models", get(llm_models_routes::get_user_models))
        .route("/api/llm/selection", put(llm_models_routes::put_user_selection))
        .route(
            "/api/personality/history",
            get(personality_routes::get_personality_history),
        )
        .route("/api/personality/rollback", post(personality_routes::rollback_personality))
        .route("/api/admin/dashboard", get(admin_dashboard_routes::get_dashboard))
        .route("/api/admin/agents", get(admin_dashboard_routes::get_agents))
        .route(
            "/api/admin/users",
            get(admin_users_routes::list_users),
        )
        .route(
            "/api/admin/users/:id",
            get(admin_users_routes::get_user_detail),
        )
        .route(
            "/api/admin/users/:id/permissions",
            post(admin_users_routes::grant_user_permission),
        )
        .route(
            "/api/admin/users/:id/permissions/:permission_id",
            delete(admin_users_routes::revoke_user_permission),
        )
        .route(
            "/api/admin/users/:id/memberships",
            post(admin_users_routes::assign_user_to_org),
        )
        .route(
            "/api/admin/users/:id/memberships/:org_id",
            delete(admin_users_routes::remove_user_from_org),
        )
        .route("/api/admin/orgs", get(admin_users_routes::list_orgs))
        .route(
            "/api/admin/settings/embedding",
            get(settings_routes::get_embedding_settings).put(settings_routes::put_embedding_settings),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
