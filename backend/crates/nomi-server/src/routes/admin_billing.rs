//! Admin → Plans → billing: the terms people can pay for, coupons, and every payment. Same
//! permission area as the plans themselves.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use nomi_auth::claims::Claims;
use nomi_auth::extractor::AuthClaims;
use nomi_usage::billing::{self, Coupon, CouponInput, DeleteCoupon, PaymentPage, Term, TermInput, STATUSES};

use crate::app::AppState;
use crate::routes::deploy::ApiError;

fn internal(e: sqlx::Error) -> ApiError {
    tracing::error!(error = %e, "billing admin query failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong".to_string())
}

fn require(claims: &Claims, action: &str) -> Result<(), ApiError> {
    if claims.has_permission("admin", "plans", action) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "not authorized to manage plans".to_string()))
    }
}

fn unique_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}

fn bad(message: &'static str) -> ApiError {
    (StatusCode::BAD_REQUEST, message.to_string())
}

/// `GET /api/admin/billing/terms`: every term, hidden ones too.
pub async fn list_terms(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Vec<Term>>, ApiError> {
    require(&claims, "view")?;
    Ok(Json(billing::list_terms(&state.pool, false).await.map_err(internal)?))
}

/// `POST /api/admin/billing/terms`.
pub async fn create_term(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(input): Json<TermInput>) -> Result<(StatusCode, Json<Term>), ApiError> {
    require(&claims, "write")?;
    let input = input.validate().map_err(bad)?;
    match billing::create_term(&state.pool, &input).await {
        Ok(term) => Ok((StatusCode::CREATED, Json(term))),
        Err(e) if unique_violation(&e) => Err((StatusCode::CONFLICT, "there's already a term for that many months".to_string())),
        Err(e) => Err(internal(e)),
    }
}

/// `PUT /api/admin/billing/terms/:id`.
pub async fn update_term(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>, Json(input): Json<TermInput>) -> Result<Json<Term>, ApiError> {
    require(&claims, "write")?;
    let input = input.validate().map_err(bad)?;
    match billing::update_term(&state.pool, id, &input).await {
        Ok(Some(term)) => Ok(Json(term)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "term not found".to_string())),
        Err(e) if unique_violation(&e) => Err((StatusCode::CONFLICT, "there's already a term for that many months".to_string())),
        Err(e) => Err(internal(e)),
    }
}

/// `DELETE /api/admin/billing/terms/:id`.
pub async fn delete_term(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    require(&claims, "manage")?;
    if billing::delete_term(&state.pool, id).await.map_err(internal)? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "term not found".to_string()))
    }
}

/// `GET /api/admin/coupons`: with how many times each was used (pending and paid payments).
pub async fn list_coupons(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Vec<Coupon>>, ApiError> {
    require(&claims, "view")?;
    Ok(Json(billing::list_coupons(&state.pool).await.map_err(internal)?))
}

/// `POST /api/admin/coupons`.
pub async fn create_coupon(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(input): Json<CouponInput>) -> Result<(StatusCode, Json<Coupon>), ApiError> {
    require(&claims, "write")?;
    let input = input.validate().map_err(bad)?;
    match billing::create_coupon(&state.pool, &input, claims.sub).await {
        Ok(coupon) => Ok((StatusCode::CREATED, Json(coupon))),
        Err(e) if unique_violation(&e) => Err((StatusCode::CONFLICT, "another coupon already uses that code".to_string())),
        Err(e) => Err(internal(e)),
    }
}

/// `PUT /api/admin/coupons/:id`.
pub async fn update_coupon(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>, Json(input): Json<CouponInput>) -> Result<Json<Coupon>, ApiError> {
    require(&claims, "write")?;
    let input = input.validate().map_err(bad)?;
    match billing::update_coupon(&state.pool, id, &input).await {
        Ok(Some(coupon)) => Ok(Json(coupon)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "coupon not found".to_string())),
        Err(e) if unique_violation(&e) => Err((StatusCode::CONFLICT, "another coupon already uses that code".to_string())),
        Err(e) => Err(internal(e)),
    }
}

/// `DELETE /api/admin/coupons/:id`: only one nobody used; deactivate the others instead.
pub async fn delete_coupon(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    require(&claims, "manage")?;
    match billing::delete_coupon(&state.pool, id).await.map_err(internal)? {
        DeleteCoupon::Deleted => Ok(StatusCode::NO_CONTENT),
        DeleteCoupon::NotFound => Err((StatusCode::NOT_FOUND, "coupon not found".to_string())),
        DeleteCoupon::Redeemed => Err((StatusCode::CONFLICT, "this coupon has been used; deactivate it instead".to_string())),
    }
}

#[derive(Deserialize)]
pub struct PaymentsQuery {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
}

/// `GET /api/admin/payments?status=&page=`: every payment with the payer's email and the plan's name.
pub async fn list_payments(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Query(query): Query<PaymentsQuery>) -> Result<Json<PaymentPage>, ApiError> {
    require(&claims, "view")?;
    let status = query.status.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if status.is_some_and(|s| !STATUSES.contains(&s)) {
        return Err(bad("unknown status"));
    }
    Ok(Json(billing::list_payments_for_admin(&state.pool, status, query.page.unwrap_or(1)).await.map_err(internal)?))
}
