//! Admin → Payment: which provider new payments go through, and each provider's keys. Secrets
//! are never sent back; a field only says whether it is set and where it comes from.

use std::collections::BTreeMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use nomi_auth::catalog::{MANAGE, VIEW};
use nomi_auth::extractor::AuthClaims;
use nomi_auth::claims::Claims;
use nomi_payment::doku::{DokuClient, DokuConfig};
use nomi_payment::xendit::XenditClient;
use nomi_payment::{PaymentError, PaymentGateway, PaymentProviderKind};
use nomi_settings::config::{self, ConfigError, FieldView, Section, SectionDef, Source};

use crate::app::AppState;
use crate::routes::admin_access::require_admin_owned;
use crate::routes::admin_configuration::SaveRequest;
use crate::routes::deploy::ApiError;

const AREA: &str = "configuration";
/// The `PAYMENT` field that names the chosen provider.
const PROVIDER_FIELD: &str = "provider";
/// An order number no payment of ours has (ours start `NM`): asking DOKU for its status proves
/// the keys sign requests DOKU accepts without touching a real order.
const DOKU_PROBE_INVOICE: &str = "NOMI-KEY-CHECK";

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

/// One provider on the overview: whether its required keys are there.
#[derive(Serialize)]
pub struct ProviderStatus {
    pub name: &'static str,
    pub configured: bool,
}

/// `GET /api/admin/payment`: the chosen provider, the one new payments actually go through, and
/// which providers are set up.
#[derive(Serialize)]
pub struct PaymentOverview {
    /// The provider chosen in `PAYMENT.provider`; `None` means "the first one set up".
    pub chosen: Option<&'static str>,
    pub source: Source,
    /// Where new payments go now; `None` when nothing is set up, or the chosen one isn't.
    pub in_use: Option<&'static str>,
    pub providers: Vec<ProviderStatus>,
    pub can_manage: bool,
}

#[derive(Deserialize)]
pub struct ChooseProviderRequest {
    /// `xendit`, `doku`, or empty to go back to the first one set up.
    #[serde(default)]
    pub provider: Option<String>,
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

fn read_failed(def: &SectionDef, e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read {} configuration: {e}", def.name))
}

fn save_failed(def: &SectionDef, e: ConfigError) -> (StatusCode, String) {
    match e {
        ConfigError::Invalid(why) => (StatusCode::UNPROCESSABLE_ENTITY, why),
        ConfigError::NoKey => (StatusCode::SERVICE_UNAVAILABLE, e.to_string()),
        ConfigError::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't save {} configuration: {e}", def.name)),
    }
}

/// A section as the server would use it: what's saved, else the environment.
async fn section(state: &AppState, def: &'static SectionDef) -> Result<Section, ApiError> {
    Ok(config::load(&state.pool, Some(&state.settings_key), def).await.map_err(|e| read_failed(def, e))?.with_env(env))
}

async fn provider_view(state: &AppState, claims: &Claims, provider: PaymentProviderKind, def: &'static SectionDef) -> Result<Json<PaymentConfigView>, ApiError> {
    let fields = config::view(&state.pool, Some(&state.settings_key), def, env).await.map_err(|e| read_failed(def, e))?;
    Ok(Json(PaymentConfigView { provider: provider.as_str(), fields, can_manage: claims.has_permission("admin", AREA, MANAGE) }))
}

async fn save_provider(state: &AppState, claims: &Claims, provider: PaymentProviderKind, def: &'static SectionDef, values: &BTreeMap<String, String>) -> Result<Json<PaymentConfigView>, ApiError> {
    require_admin_owned(claims, AREA, MANAGE)?;
    config::save(&state.pool, &state.settings_key, def, values, claims.sub).await.map_err(|e| save_failed(def, e))?;
    tracing::info!(user = %claims.sub, provider = provider.as_str(), "payment provider configuration saved");
    provider_view(state, claims, provider, def).await
}

fn xendit_ready(xendit: &Section) -> bool {
    xendit.get("secret_key").is_some_and(|k| !k.trim().is_empty())
}

fn doku_config_from(doku: &Section) -> Option<DokuConfig> {
    let present = |name: &str| doku.get(name).filter(|v| !v.trim().is_empty());
    Some(DokuConfig { client_id: present("client_id")?, secret_key: present("secret_key")?, base_url: present("base_url"), notification_path: present("notification_path") })
}

async fn overview(state: &AppState, claims: &Claims) -> Result<Json<PaymentOverview>, ApiError> {
    let fields = config::view(&state.pool, Some(&state.settings_key), &config::PAYMENT, env).await.map_err(|e| read_failed(&config::PAYMENT, e))?;
    let field = fields.iter().find(|f| f.name == PROVIDER_FIELD);
    let chosen = field.and_then(|f| f.value.as_deref()).and_then(PaymentProviderKind::from_name);
    let source = field.map_or(Source::None, |f| if chosen.is_some() { f.source } else { Source::None });

    let xendit = xendit_ready(&section(state, &config::XENDIT).await?);
    let doku = doku_config_from(&section(state, &config::DOKU).await?).is_some();
    let configured = |kind: &PaymentProviderKind| match kind {
        PaymentProviderKind::Xendit => xendit,
        PaymentProviderKind::Doku => doku,
    };
    // As `billing::gateway` picks: the chosen one if it is set up, else with none chosen the first set up.
    let in_use = match &chosen {
        Some(kind) => configured(kind).then(|| kind.as_str()),
        None => PaymentProviderKind::ALL.iter().find(|k| configured(k)).map(|k| k.as_str()),
    };
    Ok(Json(PaymentOverview {
        chosen: chosen.as_ref().map(|k| k.as_str()),
        source,
        in_use,
        providers: PaymentProviderKind::ALL.iter().map(|k| ProviderStatus { name: k.as_str(), configured: configured(k) }).collect(),
        can_manage: claims.has_permission("admin", AREA, MANAGE),
    }))
}

/// `GET /api/admin/payment`: the default provider and which providers are set up.
pub async fn get_overview(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<PaymentOverview>, ApiError> {
    require_admin_owned(&claims, AREA, VIEW)?;
    overview(&state, &claims).await
}

/// `PUT /api/admin/payment`: chooses the provider new payments go through.
pub async fn choose_provider(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<ChooseProviderRequest>) -> Result<Json<PaymentOverview>, ApiError> {
    require_admin_owned(&claims, AREA, MANAGE)?;
    let values = BTreeMap::from([(PROVIDER_FIELD.to_string(), req.provider.unwrap_or_default())]);
    config::save(&state.pool, &state.settings_key, &config::PAYMENT, &values, claims.sub).await.map_err(|e| save_failed(&config::PAYMENT, e))?;
    tracing::info!(user = %claims.sub, "default payment provider saved");
    overview(&state, &claims).await
}

/// `GET /api/admin/payment/xendit`: returns the current Xendit settings fields and status.
pub async fn get_xendit(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<PaymentConfigView>, ApiError> {
    require_admin_owned(&claims, AREA, VIEW)?;
    provider_view(&state, &claims, PaymentProviderKind::Xendit, &config::XENDIT).await
}

/// `PUT /api/admin/payment/xendit`: saves updated Xendit settings.
pub async fn save_xendit(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<SaveRequest>) -> Result<Json<PaymentConfigView>, ApiError> {
    save_provider(&state, &claims, PaymentProviderKind::Xendit, &config::XENDIT, &req.values).await
}

/// `POST /api/admin/payment/xendit/test`: validates the configured Xendit API key.
pub async fn test_xendit(AuthClaims(claims): AuthClaims) -> Result<Json<TestConnectionResponse>, ApiError> {
    require_admin_owned(&claims, AREA, VIEW)?;

    let xendit_cfg = crate::configuration::xendit_config().await;
    match xendit_cfg {
        Some(cfg) => match XenditClient::new(cfg) {
            Ok(_) => Ok(Json(TestConnectionResponse { success: true, message: "Xendit client initialized successfully".to_string() })),
            Err(e) => Ok(Json(TestConnectionResponse { success: false, message: format!("Configuration error: {e}") })),
        },
        None => Ok(Json(TestConnectionResponse { success: false, message: "Xendit secret key is not set".to_string() })),
    }
}

/// `GET /api/admin/payment/doku`: the DOKU Checkout settings, secrets as set/unset only.
pub async fn get_doku(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<PaymentConfigView>, ApiError> {
    require_admin_owned(&claims, AREA, VIEW)?;
    provider_view(&state, &claims, PaymentProviderKind::Doku, &config::DOKU).await
}

/// `PUT /api/admin/payment/doku`: saves the DOKU Checkout settings.
pub async fn save_doku(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<SaveRequest>) -> Result<Json<PaymentConfigView>, ApiError> {
    save_provider(&state, &claims, PaymentProviderKind::Doku, &config::DOKU, &req.values).await
}

/// What asking DOKU for an order's status says about the keys: anything but a refused
/// signature means DOKU accepted them.
fn doku_probe_result(result: Result<(), PaymentError>) -> TestConnectionResponse {
    match result {
        Ok(()) | Err(PaymentError::NotFound(_)) => TestConnectionResponse { success: true, message: "DOKU accepted the client ID and secret key".to_string() },
        Err(PaymentError::Unauthorized(why)) => TestConnectionResponse { success: false, message: format!("DOKU refused the client ID and secret key: {why}") },
        Err(e) => TestConnectionResponse { success: false, message: format!("Couldn't check the keys with DOKU: {e}") },
    }
}

/// `POST /api/admin/payment/doku/test`: signs a status request for an order that doesn't exist
/// with the saved keys; DOKU answers "not found" for good keys and "unauthorized" for bad ones.
pub async fn test_doku(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<TestConnectionResponse>, ApiError> {
    require_admin_owned(&claims, AREA, VIEW)?;
    let Some(cfg) = doku_config_from(&section(&state, &config::DOKU).await?) else {
        return Ok(Json(TestConnectionResponse { success: false, message: "DOKU client ID and secret key are not set".to_string() }));
    };
    let client = match DokuClient::new(cfg) {
        Ok(client) => client,
        Err(e) => return Ok(Json(TestConnectionResponse { success: false, message: format!("Configuration error: {e}") })),
    };
    Ok(Json(doku_probe_result(client.get_invoice(DOKU_PROBE_INVOICE).await.map(|_| ()))))
}

#[cfg(test)]
mod tests {
    use super::*;

    const GATEWAY_DOWN: u16 = 503;

    #[test]
    fn a_missing_order_means_the_keys_were_accepted() {
        assert!(doku_probe_result(Err(PaymentError::NotFound("no order".into()))).success);
        assert!(doku_probe_result(Ok(())).success);
    }

    #[test]
    fn a_refused_signature_or_an_unreachable_doku_fails_the_test() {
        let refused = doku_probe_result(Err(PaymentError::Unauthorized("invalid signature".into())));
        assert!(!refused.success);
        assert!(refused.message.contains("refused"));
        assert!(!doku_probe_result(Err(PaymentError::ApiError { status: GATEWAY_DOWN, message: "down".into() })).success);
    }
}
