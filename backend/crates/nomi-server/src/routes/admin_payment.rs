use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use nomi_auth::catalog::{MANAGE, VIEW};
use nomi_auth::extractor::AuthClaims;
use nomi_payment::xendit::XenditClient;
use nomi_settings::config::{self, ConfigError, FieldView};

use crate::app::AppState;
use crate::routes::admin_access::require_admin_owned;
use crate::routes::admin_configuration::SaveRequest;
use crate::routes::deploy::ApiError;

const AREA: &str = "configuration";

#[derive(Serialize)]
pub struct PaymentConfigView {
    pub provider: &'static str,
    pub fields: Vec<FieldView>,
    pub can_manage: bool,
}

#[derive(Serialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// `GET /api/admin/payment/xendit`: returns the current Xendit settings fields and status.
pub async fn get_xendit(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<PaymentConfigView>, ApiError> {
    require_admin_owned(&claims, AREA, VIEW)?;
    let fields = config::view(&state.pool, Some(&state.settings_key), &config::XENDIT, env)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read Xendit configuration: {e}")))?;

    Ok(Json(PaymentConfigView {
        provider: "xendit",
        fields,
        can_manage: claims.has_permission("admin", AREA, MANAGE),
    }))
}

/// `PUT /api/admin/payment/xendit`: saves updated Xendit settings.
pub async fn save_xendit(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<SaveRequest>,
) -> Result<Json<PaymentConfigView>, ApiError> {
    require_admin_owned(&claims, AREA, MANAGE)?;

    config::save(
        &state.pool,
        &state.settings_key,
        &config::XENDIT,
        &req.values,
        claims.sub,
    )
    .await
    .map_err(|e| match e {
        ConfigError::Invalid(why) => (StatusCode::UNPROCESSABLE_ENTITY, why),
        ConfigError::NoKey => (StatusCode::SERVICE_UNAVAILABLE, e.to_string()),
        ConfigError::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't save Xendit configuration: {e}")),
    })?;

    tracing::info!(user = %claims.sub, "Xendit configuration saved");

    let fields = config::view(&state.pool, Some(&state.settings_key), &config::XENDIT, env)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read Xendit configuration: {e}")))?;

    Ok(Json(PaymentConfigView {
        provider: "xendit",
        fields,
        can_manage: claims.has_permission("admin", AREA, MANAGE),
    }))
}

/// `POST /api/admin/payment/xendit/test`: validates the configured Xendit API key.
pub async fn test_xendit(
    AuthClaims(claims): AuthClaims,
) -> Result<Json<TestConnectionResponse>, ApiError> {
    require_admin_owned(&claims, AREA, VIEW)?;

    let xendit_cfg = crate::configuration::xendit_config().await;
    match xendit_cfg {
        Some(cfg) => match XenditClient::new(cfg) {
            Ok(_) => Ok(Json(TestConnectionResponse {
                success: true,
                message: "Xendit client initialized successfully".to_string(),
            })),
            Err(e) => Ok(Json(TestConnectionResponse {
                success: false,
                message: format!("Configuration error: {e}"),
            })),
        },
        None => Ok(Json(TestConnectionResponse {
            success: false,
            message: "Xendit secret key is not set".to_string(),
        })),
    }
}
