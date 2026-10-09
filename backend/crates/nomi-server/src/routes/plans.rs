//! Plans and subscriptions: the plans people can see (`/api/plans`), Admin → Plans, and an admin
//! changing someone's plan or allowance (which notifies them).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::claims::Claims;
use nomi_auth::extractor::AuthClaims;
use nomi_i18n::Locale;
use nomi_usage::plans::{self, ChangeRow, DeletePlan, PlanInput, PlanRow, SubscriptionChange};

use crate::app::AppState;
use crate::notifications::{self, Kind, Notice};
use crate::quota::format_tokens;

type ApiError = (StatusCode, &'static str);

fn internal(e: sqlx::Error) -> ApiError {
    tracing::error!(error = %e, "plans query failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
}

fn require(claims: &Claims, action: &str) -> Result<(), ApiError> {
    if claims.has_permission("admin", "user", action) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "not authorized to manage plans"))
    }
}

#[derive(Serialize)]
pub struct PlansForUser {
    pub plans: Vec<PlanRow>,
    /// The id of the plan they're on.
    pub current_plan_id: Uuid,
    pub monthly_tokens: i64,
    pub custom_quota: bool,
    pub override_until: Option<DateTime<Utc>>,
}

/// `GET /api/plans`: the plans on offer, and which one they're on.
pub async fn list_for_user(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<PlansForUser>, ApiError> {
    let subscription = plans::subscription_for(&state.pool, claims.sub).await.map_err(internal)?;
    let mut list = plans::list_plans(&state.pool, true).await.map_err(internal)?;
    // A promo past its end date isn't on offer any more.
    for plan in &mut list {
        if plan.promo_ends_at.is_some_and(|end| end <= Utc::now()) {
            plan.promo_label = None;
            plan.promo_price_label = None;
        }
    }
    Ok(Json(PlansForUser {
        plans: list,
        current_plan_id: subscription.plan.id,
        monthly_tokens: subscription.monthly_tokens,
        custom_quota: subscription.quota_override.is_some(),
        override_until: subscription.override_until,
    }))
}

#[derive(Serialize)]
pub struct AdminPlan {
    #[serde(flatten)]
    pub plan: PlanRow,
    /// How many people are on it (the default plan counts everyone with no subscription).
    pub subscribers: i64,
}

/// `GET /api/admin/plans`: every plan, hidden ones too.
pub async fn admin_list(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Vec<AdminPlan>>, ApiError> {
    require(&claims, "view")?;
    let list = plans::list_plans(&state.pool, false).await.map_err(internal)?;
    let mut out = Vec::with_capacity(list.len());
    for plan in list {
        let subscribers: i64 = if plan.is_default {
            sqlx::query_scalar(
                "SELECT count(*) FROM users u WHERE NOT EXISTS (SELECT 1 FROM user_subscriptions s WHERE s.user_id = u.id AND s.plan_id <> $1)",
            )
        } else {
            sqlx::query_scalar("SELECT count(*) FROM user_subscriptions WHERE plan_id = $1")
        }
        .bind(plan.id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;
        out.push(AdminPlan { plan, subscribers });
    }
    Ok(Json(out))
}

fn unique_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}

/// `POST /api/admin/plans`.
pub async fn admin_create(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(input): Json<PlanInput>) -> Result<(StatusCode, Json<PlanRow>), ApiError> {
    require(&claims, "manage")?;
    let input = input.validate().map_err(|m| (StatusCode::BAD_REQUEST, m))?;
    match plans::create_plan(&state.pool, &input).await {
        Ok(plan) => Ok((StatusCode::CREATED, Json(plan))),
        Err(e) if unique_violation(&e) => Err((StatusCode::CONFLICT, "another plan already uses that slug")),
        Err(e) => Err(internal(e)),
    }
}

/// `PUT /api/admin/plans/:id`.
pub async fn admin_update(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(input): Json<PlanInput>,
) -> Result<Json<PlanRow>, ApiError> {
    require(&claims, "manage")?;
    let input = input.validate().map_err(|m| (StatusCode::BAD_REQUEST, m))?;
    match plans::update_plan(&state.pool, id, &input).await {
        Ok(Some(plan)) => Ok(Json(plan)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "plan not found")),
        Err(e) if unique_violation(&e) => Err((StatusCode::CONFLICT, "another plan already uses that slug")),
        Err(e) => Err(internal(e)),
    }
}

/// `PUT /api/admin/plans/:id/default`: where new people start.
pub async fn admin_set_default(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    require(&claims, "manage")?;
    match plans::set_default_plan(&state.pool, id).await.map_err(internal)? {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err((StatusCode::NOT_FOUND, "plan not found")),
    }
}

/// `DELETE /api/admin/plans/:id`: only a plan nobody was ever on; hide the others instead.
pub async fn admin_delete(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    require(&claims, "manage")?;
    match plans::delete_plan(&state.pool, id).await.map_err(internal)? {
        DeletePlan::Deleted => Ok(StatusCode::NO_CONTENT),
        DeletePlan::NotFound => Err((StatusCode::NOT_FOUND, "plan not found")),
        DeletePlan::IsDefault => Err((StatusCode::BAD_REQUEST, "make another plan the default first")),
        DeletePlan::InUse => Err((StatusCode::CONFLICT, "people are or were on this plan; hide it instead")),
    }
}

#[derive(Serialize)]
pub struct UserSubscription {
    pub plan: PlanRow,
    pub quota_override: Option<i64>,
    pub override_until: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub monthly_tokens: i64,
    /// This month, in their timezone.
    pub month: String,
    pub tokens_used: i64,
    pub history: Vec<ChangeRow>,
}

async fn load_user_subscription(state: &AppState, user_id: Uuid) -> Result<UserSubscription, ApiError> {
    let subscription = plans::subscription_for(&state.pool, user_id).await.map_err(internal)?;
    let brief = nomi_usage::brief(&state.pool, user_id).await.map_err(internal)?;
    let history = plans::change_history(&state.pool, user_id).await.map_err(internal)?;
    Ok(UserSubscription {
        plan: subscription.plan,
        quota_override: subscription.quota_override,
        override_until: subscription.override_until,
        note: subscription.note,
        monthly_tokens: subscription.monthly_tokens,
        month: brief.month,
        tokens_used: brief.tokens_used,
        history,
    })
}

async fn require_user_exists(state: &AppState, user_id: Uuid) -> Result<(), ApiError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)").bind(user_id).fetch_one(&state.pool).await.map_err(internal)?;
    if exists {
        Ok(())
    } else {
        Err((StatusCode::NOT_FOUND, "user not found"))
    }
}

/// `GET /api/admin/users/:id/subscription`: their plan, allowance, this month's use and history.
pub async fn admin_get_subscription(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(user_id): Path<Uuid>) -> Result<Json<UserSubscription>, ApiError> {
    require(&claims, "view")?;
    require_user_exists(&state, user_id).await?;
    Ok(Json(load_user_subscription(&state, user_id).await?))
}

#[derive(Deserialize)]
pub struct SubscriptionRequest {
    pub plan_id: Uuid,
    pub quota_override: Option<i64>,
    pub override_until: Option<DateTime<Utc>>,
    pub note: Option<String>,
}

/// What the person is told about a change: a new plan, or a new allowance on the same one.
fn change_notice(locale: Locale, before: &plans::Subscription, after: &plans::Subscription, note: Option<&str>) -> Notice {
    let quota = format_tokens(after.monthly_tokens, locale);
    let (title, mut body) = if before.plan.id != after.plan.id {
        (
            locale.tf("subscription.plan_title", &[("plan", &after.plan.name)]),
            locale.tf("subscription.plan_body", &[("plan", &after.plan.name), ("quota", &quota)]),
        )
    } else {
        let until = match after.override_until {
            Some(date) => locale.tf("subscription.until", &[("date", &date.format("%Y-%m-%d").to_string())]),
            None => String::new(),
        };
        (
            locale.tf("subscription.quota_title", &[("quota", &quota)]),
            locale.tf("subscription.quota_body", &[("quota", &quota), ("until", &until)]),
        )
    };
    if let Some(note) = note.map(str::trim).filter(|n| !n.is_empty()) {
        body.push_str("\n\n");
        body.push_str(&locale.tf("subscription.note", &[("note", note)]));
    }
    Notice { kind: Kind::Subscription, title, body, link: Some("/billing".to_string()) }
}

/// `PUT /api/admin/users/:id/subscription`: moves someone to a plan, and/or overrides its
/// allowance. They're notified in the app and by email.
pub async fn admin_put_subscription(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(user_id): Path<Uuid>,
    Json(req): Json<SubscriptionRequest>,
) -> Result<Json<UserSubscription>, ApiError> {
    require(&claims, "manage")?;
    require_user_exists(&state, user_id).await?;
    if req.quota_override.is_some_and(|q| q < 0) {
        return Err((StatusCode::BAD_REQUEST, "quota must be zero or more"));
    }
    if req.override_until.is_some_and(|until| until <= Utc::now()) {
        return Err((StatusCode::BAD_REQUEST, "the override must end in the future"));
    }
    if plans::get_plan(&state.pool, req.plan_id).await.map_err(internal)?.is_none() {
        return Err((StatusCode::BAD_REQUEST, "unknown plan"));
    }

    let before = plans::subscription_for(&state.pool, user_id).await.map_err(internal)?;
    let change = SubscriptionChange { plan_id: req.plan_id, quota_override: req.quota_override, override_until: req.override_until, note: req.note.clone() };
    plans::change_subscription(&state.pool, user_id, &change, claims.sub).await.map_err(internal)?;
    let after = plans::subscription_for(&state.pool, user_id).await.map_err(internal)?;

    if before.plan.id != after.plan.id || before.monthly_tokens != after.monthly_tokens || before.override_until != after.override_until {
        let language: Option<String> = sqlx::query_scalar("SELECT language FROM user_preferences WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal)?;
        let locale = Locale::from_code_or_default(language.as_deref());
        notifications::send(&state.pool, user_id, change_notice(locale, &before, &after, req.note.as_deref())).await.map_err(internal)?;
    }
    Ok(Json(load_user_subscription(&state, user_id).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(name: &str, tokens: i64) -> PlanRow {
        PlanRow {
            id: Uuid::new_v4(),
            slug: name.to_lowercase(),
            name: name.into(),
            description: String::new(),
            monthly_tokens: tokens,
            price_label: String::new(),
            features: vec![],
            card_tone: "glow".into(),
            promo_label: None,
            promo_price_label: None,
            promo_ends_at: None,
            is_default: false,
            is_active: true,
            sort_order: 0,
        }
    }

    fn subscription(plan: PlanRow, monthly_tokens: i64) -> plans::Subscription {
        plans::Subscription { plan, quota_override: None, override_until: None, note: None, monthly_tokens }
    }

    #[test]
    fn says_what_changed_in_their_language() {
        let free = plan("Free", 1_000_000);
        let pro = plan("Pro", 10_000_000);
        let moved = change_notice(Locale::En, &subscription(free.clone(), 1_000_000), &subscription(pro, 10_000_000), Some("Thanks for testing!"));
        assert_eq!(moved.title, "You're on Pro now");
        assert!(moved.body.contains("10,000,000 tokens"));
        assert!(moved.body.contains("Thanks for testing!"));

        let raised = change_notice(Locale::Id, &subscription(free.clone(), 1_000_000), &subscription(free, 3_000_000), None);
        assert!(raised.title.contains("3.000.000"), "{}", raised.title);
    }
}
