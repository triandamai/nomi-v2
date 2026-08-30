use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct ProfileResponse {
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub avatar_url: Option<String>,
}

async fn load_email(pool: &sqlx::PgPool, user_id: Uuid) -> Result<String, (StatusCode, &'static str)> {
    sqlx::query_scalar("SELECT email FROM web_credentials WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to load email for profile");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to load profile")
        })
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_profile(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<ProfileResponse>, (StatusCode, &'static str)> {
    let email = load_email(&state.pool, claims.sub).await?;

    let row: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT display_name, username, avatar_url FROM user_profiles WHERE user_id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to load profile");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load profile")
    })?;

    let (display_name, username, avatar_url) = row.unwrap_or((None, None, None));

    Ok(Json(ProfileResponse { display_name, username, email, avatar_url }))
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[tracing::instrument(skip(state, claims, req))]
pub async fn put_profile(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, (StatusCode, String)> {
    if let Some(username) = &req.username {
        if !username.is_empty() && !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
            return Err((
                StatusCode::BAD_REQUEST,
                "username may only contain letters, numbers, underscores, and periods".to_string(),
            ));
        }
    }

    sqlx::query(
        "INSERT INTO user_profiles (user_id, display_name, username, avatar_url) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (user_id) DO UPDATE SET \
             display_name = COALESCE($2, user_profiles.display_name), \
             username = COALESCE($3, user_profiles.username), \
             avatar_url = COALESCE($4, user_profiles.avatar_url), \
             updated_at = now()",
    )
    .bind(claims.sub)
    .bind(req.display_name.as_deref().filter(|s| !s.is_empty()))
    .bind(req.username.as_deref().filter(|s| !s.is_empty()))
    .bind(req.avatar_url.as_deref())
    .execute(&state.pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.is_unique_violation() {
                return (StatusCode::CONFLICT, "that username is already taken".to_string());
            }
        }
        tracing::error!(error = %e, "failed to save profile");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save profile".to_string())
    })?;

    let email = load_email(&state.pool, claims.sub).await.map_err(|(status, msg)| (status, msg.to_string()))?;
    let row: (Option<String>, Option<String>, Option<String>) =
        sqlx::query_as("SELECT display_name, username, avatar_url FROM user_profiles WHERE user_id = $1")
            .bind(claims.sub)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to reload profile");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to reload profile".to_string())
            })?;

    Ok(Json(ProfileResponse { display_name: row.0, username: row.1, email, avatar_url: row.2 }))
}

#[derive(Deserialize)]
pub struct AvatarUploadUrlRequest {
    pub content_type: String,
}

#[derive(Serialize)]
pub struct AvatarUploadUrlResponse {
    pub upload_url: String,
    pub public_url: String,
}

const ALLOWED_AVATAR_CONTENT_TYPES: [&str; 4] = ["image/png", "image/jpeg", "image/webp", "image/gif"];

#[tracing::instrument(skip(state, claims, req))]
pub async fn request_avatar_upload_url(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<AvatarUploadUrlRequest>,
) -> Result<Json<AvatarUploadUrlResponse>, (StatusCode, String)> {
    let Some(s3) = &state.s3 else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "avatar upload is not configured".to_string()));
    };

    if !ALLOWED_AVATAR_CONTENT_TYPES.contains(&req.content_type.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "content_type must be an image/png, image/jpeg, image/webp, or image/gif".to_string()));
    }
    let extension = match req.content_type.as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => "gif",
    };

    let key = format!("avatars/{}/{}.{extension}", claims.sub, Uuid::new_v4());
    let presigned = s3.presign_put(&key, &req.content_type).await.map_err(|e| {
        tracing::error!(error = %e, "failed to presign avatar upload url");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to prepare upload".to_string())
    })?;

    Ok(Json(AvatarUploadUrlResponse { upload_url: presigned.upload_url, public_url: presigned.public_url }))
}

#[derive(Serialize)]
pub struct PreferencesResponse {
    pub theme: String,
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_preferences(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<PreferencesResponse>, (StatusCode, &'static str)> {
    let theme: Option<String> = sqlx::query_scalar("SELECT theme FROM user_preferences WHERE user_id = $1")
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to load preferences");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to load preferences")
        })?;

    Ok(Json(PreferencesResponse { theme: theme.unwrap_or_else(|| "system".to_string()) }))
}

#[derive(Deserialize)]
pub struct UpdatePreferencesRequest {
    pub theme: String,
}

const ALLOWED_THEMES: [&str; 3] = ["light", "dark", "system"];

#[tracing::instrument(skip(state, claims, req))]
pub async fn put_preferences(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<UpdatePreferencesRequest>,
) -> Result<Json<PreferencesResponse>, (StatusCode, &'static str)> {
    if !ALLOWED_THEMES.contains(&req.theme.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "theme must be 'light', 'dark', or 'system'"));
    }

    sqlx::query(
        "INSERT INTO user_preferences (user_id, theme) VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE SET theme = EXCLUDED.theme, updated_at = now()",
    )
    .bind(claims.sub)
    .bind(&req.theme)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to save preferences");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences")
    })?;

    Ok(Json(PreferencesResponse { theme: req.theme }))
}
