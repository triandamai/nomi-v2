//! `POST /api/attachments/estimate`: before sending, what reading the attached files would cost on
//! the model the person picked: tokens per file and in all, who pays (Nomi's allowance or their
//! own key), the price in rupiah when it's Nomi's, and the size past which the composer warns.
//! The sums are plain functions; the handler only looks things up.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_llm::catalog;
use nomi_settings::model_choices::ModelKey;
use nomi_settings::provider_connections::{self as connections, Scope};
use nomi_settings::llm_models;

use crate::app::AppState;

/// What a model reads an image, or a page read as an image, for (about what Claude and GPT take
/// an image at the size the preview is kept at).
const TOKENS_PER_IMAGE: u64 = 1_600;
/// The warning appears when the files fill this share of the model's context window.
const THRESHOLD_CONTEXT_PERCENT: u64 = 50;
/// The warning's size when the model's context window isn't known.
const DEFAULT_THRESHOLD_TOKENS: u64 = 50_000;
const TOKENS_PER_MILLION: f64 = 1_000_000.0;
const PERCENT: u64 = 100;

/// Who pays for reading the files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Payer {
    /// Nomi's own model, counted against the plan's allowance.
    Credits,
    /// A model on the person's own key.
    Byok,
}

/// Pages and tokens for one file, from what ingestion read out of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FileEstimate {
    pub pages: u64,
    pub tokens: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PlanShare {
    pub used_pct: u64,
    pub after_pct: u64,
}

/// A file's pages (pages, slides or sheets, else one) and the tokens reading it takes: the
/// extracted text at about four characters a token, or a fixed amount per image or per scanned
/// page (read as an image); audio and video aren't counted.
pub fn estimate_file(kind: &str, text: Option<&str>, details: &Value) -> FileEstimate {
    let pages = ["pages", "slides"]
        .iter()
        .find_map(|key| details.get(*key).and_then(Value::as_u64))
        .or_else(|| details.get("sheets").and_then(Value::as_array).map(|sheets| sheets.len() as u64))
        .unwrap_or(1);
    let scanned = details.get("scanned").and_then(Value::as_bool) == Some(true);
    let tokens = match kind {
        "image" => TOKENS_PER_IMAGE,
        _ if scanned => pages * TOKENS_PER_IMAGE,
        "audio" | "voice" | "video" => 0,
        _ => text.map_or(0, |t| u64::from(nomi_embedding::estimate_tokens(t))),
    };
    FileEstimate { pages, tokens }
}

/// Who pays when the model is `key`.
pub fn payer_for(key: &ModelKey) -> Payer {
    match key {
        ModelKey::Admin(_) => Payer::Credits,
        ModelKey::Connection { .. } => Payer::Byok,
    }
}

/// The price in whole rupiah (rounded up), when the model has a price and Nomi is the one paying.
pub fn cost_idr(payer: Payer, tokens: u64, input_usd_per_mtok: Option<f64>, usd_to_idr: u64) -> Option<u64> {
    if payer != Payer::Credits {
        return None;
    }
    let usd_per_mtok = input_usd_per_mtok?;
    Some((usd_per_mtok * usd_to_idr as f64 * tokens as f64 / TOKENS_PER_MILLION).ceil() as u64)
}

/// The size past which the composer warns: half the model's context window, else a default.
pub fn threshold_tokens(context: Option<u64>) -> u64 {
    match context.filter(|c| *c > 0) {
        Some(context) => context * THRESHOLD_CONTEXT_PERCENT / PERCENT,
        None => DEFAULT_THRESHOLD_TOKENS,
    }
}

/// How much of the month's allowance is used, and would be after reading the files.
pub fn plan_share(used: i64, tokens: u64, monthly: i64) -> Option<PlanShare> {
    if monthly <= 0 {
        return None;
    }
    let (used, monthly) = (used.max(0) as u64, monthly as u64);
    Some(PlanShare { used_pct: used * PERCENT / monthly, after_pct: (used + tokens) * PERCENT / monthly })
}

#[derive(Deserialize)]
pub struct EstimateRequest {
    pub attachment_ids: Vec<Uuid>,
    pub model_key: String,
}

#[derive(Serialize)]
struct FileLine {
    id: Uuid,
    #[serde(flatten)]
    estimate: FileEstimate,
}

type RouteError = (StatusCode, Json<Value>);

fn refused(status: StatusCode, code: &str) -> RouteError {
    (status, Json(json!({ "error": code })))
}

fn server_error(what: &str, e: impl std::fmt::Display) -> RouteError {
    tracing::error!(error = %e, "{what}");
    refused(StatusCode::INTERNAL_SERVER_ERROR, "server_error")
}

/// The model's provider as the person sees it, its context window and (for Nomi's models) its
/// input price.
struct Chosen {
    provider_label: String,
    context: Option<u64>,
    input_usd_per_mtok: Option<f64>,
}

async fn choose(state: &AppState, user_id: Uuid, key: &ModelKey) -> Result<Chosen, RouteError> {
    match key {
        ModelKey::Admin(id) => {
            let model = llm_models::get_admin_llm_model(&state.pool, *id)
                .await
                .map_err(|e| server_error("failed to load a model", e))?
                .filter(|m| m.enabled)
                .ok_or_else(|| refused(StatusCode::NOT_FOUND, "unknown_model"))?;
            let shared: std::collections::HashMap<Uuid, _> = connections::list_connections(&state.pool, Scope::Admin)
                .await
                .map_err(|e| server_error("failed to load providers", e))?
                .into_iter()
                .map(|c| (c.id, c))
                .collect();
            let (provider_id, provider_label) = super::model_options::admin_provider(&model, &shared);
            Ok(Chosen { context: catalog::traits(&provider_id, &model.model_id).context, provider_label, input_usd_per_mtok: model.input_usd_per_mtok })
        }
        ModelKey::Connection { connection_id, model_id } => {
            let connection = connections::get_connection(&state.pool, Scope::User(user_id), *connection_id)
                .await
                .map_err(|e| server_error("failed to load a provider", e))?
                .ok_or_else(|| refused(StatusCode::NOT_FOUND, "unknown_model"))?;
            Ok(Chosen {
                context: catalog::traits(&connection.provider_id, model_id).context,
                provider_label: super::model_options::connection_name(&connection),
                input_usd_per_mtok: None,
            })
        }
    }
}

/// `POST /api/attachments/estimate`: what reading these files costs on this model.
pub async fn estimate(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(body): Json<EstimateRequest>) -> Result<Json<Value>, RouteError> {
    let key = ModelKey::parse(&body.model_key).ok_or_else(|| refused(StatusCode::BAD_REQUEST, "invalid_model_key"))?;
    let mut ids = body.attachment_ids.clone();
    ids.sort();
    ids.dedup();
    if ids.len() > super::attachments::current_limits().await.max_files {
        return Err(refused(StatusCode::BAD_REQUEST, "too_many_files"));
    }
    let rows: Vec<(Uuid, String, Option<String>, Value)> = sqlx::query_as("SELECT id, kind, extracted_text, details FROM attachments WHERE user_id = $1 AND id = ANY($2)")
        .bind(claims.sub)
        .bind(&ids)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| server_error("failed to load files", e))?;
    if rows.len() != ids.len() {
        return Err(refused(StatusCode::NOT_FOUND, "unknown_file"));
    }
    let chosen = choose(&state, claims.sub, &key).await?;

    // In the order the person gave them.
    let files: Vec<FileLine> = body
        .attachment_ids
        .iter()
        .filter_map(|id| rows.iter().find(|row| row.0 == *id))
        .map(|(id, kind, text, details)| FileLine { id: *id, estimate: estimate_file(kind, text.as_deref(), details) })
        .collect();
    let tokens: u64 = files.iter().map(|f| f.estimate.tokens).sum();
    let payer = payer_for(&key);

    let mut out = json!({
        "files": files,
        "tokens": tokens,
        "payer": payer,
        "provider_label": chosen.provider_label,
        "threshold_tokens": threshold_tokens(chosen.context),
    });
    if let Some(idr) = cost_idr(payer, tokens, chosen.input_usd_per_mtok, super::usage::usd_to_idr().await) {
        out["cost_idr"] = json!(idr);
    }
    if payer == Payer::Credits {
        let brief = nomi_usage::brief(&state.pool, claims.sub).await.map_err(|e| server_error("failed to load the allowance", e))?;
        if let Some(share) = plan_share(brief.tokens_used, tokens, brief.plan.monthly_tokens) {
            out["plan"] = json!(share);
        }
    }
    Ok(Json(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_about_four_characters_a_token_and_pages_come_from_the_file() {
        let text = "x".repeat(4_000);
        let pdf = estimate_file("pdf", Some(&text), &json!({ "pages": 7, "scanned": false }));
        assert_eq!(pdf, FileEstimate { pages: 7, tokens: 1_000 });
        let slides = estimate_file("presentation", Some(&text), &json!({ "slides": 12 }));
        assert_eq!(slides.pages, 12);
        let sheets = estimate_file("spreadsheet", Some(&text), &json!({ "sheets": [{ "rows": 3 }, { "rows": 4 }] }));
        assert_eq!(sheets.pages, 2);
        assert_eq!(estimate_file("text", Some("hello world"), &json!({ "lines": 1 })), FileEstimate { pages: 1, tokens: 3 });
    }

    #[test]
    fn images_and_scanned_pages_cost_a_fixed_amount_each() {
        assert_eq!(estimate_file("image", Some("a description"), &json!({ "width": 800, "height": 600 })), FileEstimate { pages: 1, tokens: TOKENS_PER_IMAGE });
        assert_eq!(estimate_file("pdf", None, &json!({ "pages": 5, "scanned": true })), FileEstimate { pages: 5, tokens: 5 * TOKENS_PER_IMAGE });
        // A scanned PDF a model already transcribed is still read as pages.
        assert_eq!(estimate_file("pdf", Some("transcript"), &json!({ "pages": 2, "scanned": true })).tokens, 2 * TOKENS_PER_IMAGE);
        assert_eq!(estimate_file("audio", None, &json!({})), FileEstimate { pages: 1, tokens: 0 });
        assert_eq!(estimate_file("pdf", None, &json!({ "encrypted": true, "pages": 0 })), FileEstimate { pages: 0, tokens: 0 });
    }

    #[test]
    fn nomis_models_are_paid_in_credits_and_the_persons_own_key_is_byok() {
        let admin = ModelKey::Admin(Uuid::nil());
        let own = ModelKey::Connection { connection_id: Uuid::nil(), model_id: "gpt-5".into() };
        assert_eq!((payer_for(&admin), payer_for(&own)), (Payer::Credits, Payer::Byok));
        assert_eq!(serde_json::to_value(Payer::Byok).unwrap(), json!("byok"));
    }

    #[test]
    fn the_price_is_in_rupiah_rounded_up_and_only_when_nomi_is_paid() {
        // 250k tokens at $3 per million is $0.75; at Rp 16,000 to the dollar, Rp 12,000.
        assert_eq!(cost_idr(Payer::Credits, 250_000, Some(3.0), 16_000), Some(12_000));
        assert_eq!(cost_idr(Payer::Credits, 1, Some(3.0), 16_000), Some(1));
        assert_eq!(cost_idr(Payer::Credits, 0, Some(3.0), 16_000), Some(0));
        assert_eq!(cost_idr(Payer::Credits, 1_000, None, 16_000), None);
        assert_eq!(cost_idr(Payer::Byok, 250_000, Some(3.0), 16_000), None);
    }

    #[test]
    fn the_warning_is_half_the_context_window_or_a_default() {
        assert_eq!(threshold_tokens(Some(200_000)), 100_000);
        assert_eq!(threshold_tokens(None), DEFAULT_THRESHOLD_TOKENS);
        assert_eq!(threshold_tokens(Some(0)), DEFAULT_THRESHOLD_TOKENS);
    }

    #[test]
    fn the_plan_share_shows_the_month_before_and_after() {
        assert_eq!(plan_share(250_000, 150_000, 1_000_000), Some(PlanShare { used_pct: 25, after_pct: 40 }));
        assert_eq!(plan_share(990_000, 500_000, 1_000_000), Some(PlanShare { used_pct: 99, after_pct: 149 }));
        assert_eq!(plan_share(0, 10, 0), None);
    }
}
