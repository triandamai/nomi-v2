use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;
use nomi_i18n::Locale;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowcaseProject {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub deploy_url: Option<String>,
    pub build_duration_secs: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStats {
    pub lifetime_tokens: i64,
    pub peak_day_tokens: i64,
    pub peak_day_date: Option<String>,
    pub longest_task_secs: Option<i64>,
    pub longest_task_name: Option<String>,
    pub longest_streak_days: i32,
    pub longest_streak_range: Option<String>,
    pub current_streak_days: i32,
    pub best_streak_target: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDayActivity {
    pub date: String,
    pub tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileModelUsage {
    pub label: String,
    pub tokens: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCrewUsage {
    pub agent_id: String,
    pub name: String,
    pub count: i64,
    pub kind: String,
}

#[derive(Serialize)]
pub struct ProfileResponse {
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub plan_name: String,
    pub show_stats_publicly: bool,
    pub show_crew_publicly: bool,
    pub showcase_project_ids: Vec<Uuid>,
    pub showcase_projects: Vec<ShowcaseProject>,
    pub stats: ProfileStats,
    pub activity: Vec<ProfileDayActivity>,
    pub top_models: Vec<ProfileModelUsage>,
    pub crew_usage: Vec<ProfileCrewUsage>,
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

pub async fn load_profile_data(pool: &sqlx::PgPool, user_id: Uuid) -> Result<ProfileResponse, (StatusCode, &'static str)> {
    let email = load_email(pool, user_id).await.unwrap_or_default();

    let joined_at: chrono::DateTime<chrono::Utc> = sqlx::query_scalar("SELECT created_at FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to load user join date");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to load profile")
        })?
        .unwrap_or_else(chrono::Utc::now);

    #[allow(clippy::type_complexity)]
    let row: Option<(Option<String>, Option<String>, Option<String>, Option<String>, Option<bool>, Option<bool>, Option<Vec<Uuid>>)> = sqlx::query_as(
        "SELECT display_name, username, avatar_url, location, show_stats_publicly, show_crew_publicly, showcase_project_ids FROM user_profiles WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to load user profile");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load profile")
    })?;

    let (display_name, username, avatar_url, location, show_stats_publicly, show_crew_publicly, showcase_project_ids) =
        row.unwrap_or((None, None, None, None, None, None, None));

    let show_stats_publicly = show_stats_publicly.unwrap_or(true);
    let show_crew_publicly = show_crew_publicly.unwrap_or(false);
    let showcase_project_ids = showcase_project_ids.unwrap_or_default();

    let plan_name = match nomi_usage::plans::subscription_for(pool, user_id).await {
        Ok(sub) => format!("{} plan", sub.plan.name),
        Err(_) => "Free plan".to_string(),
    };

    let lifetime_tokens: i64 = sqlx::query_scalar(
        "SELECT COALESCE(ROUND(SUM(input_tokens - cached_input_tokens + output_tokens + cached_input_tokens * 0.1)), 0)::bigint FROM llm_usage WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let activity_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT to_char(created_at AT TIME ZONE COALESCE((SELECT timezone FROM user_preferences WHERE user_id = $1), 'UTC'), 'YYYY-MM-DD') AS day, \
                COALESCE(ROUND(SUM(input_tokens - cached_input_tokens + output_tokens + cached_input_tokens * 0.1)), 0)::bigint AS tokens \
         FROM llm_usage \
         WHERE user_id = $1 AND created_at >= (now() - interval '365 days') \
         GROUP BY day \
         ORDER BY day ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let activity: Vec<ProfileDayActivity> = activity_rows.into_iter().map(|(date, tokens)| ProfileDayActivity { date, tokens }).collect();

    let (peak_day_tokens, peak_day_date) = activity
        .iter()
        .max_by_key(|a| a.tokens)
        .map(|a| (a.tokens, Some(a.date.clone())))
        .unwrap_or((0, None));

    // Compute streak metrics
    let mut longest_streak_days = 0;
    let mut longest_streak_range = None;
    let mut current_streak_days = 0;

    let active_dates: Vec<chrono::NaiveDate> = activity
        .iter()
        .filter(|a| a.tokens > 0)
        .filter_map(|a| chrono::NaiveDate::parse_from_str(&a.date, "%Y-%m-%d").ok())
        .collect();

    if !active_dates.is_empty() {
        let mut cur_run = 0;
        let mut cur_start = active_dates[0];
        let mut prev = active_dates[0];

        for &date in &active_dates {
            if date == prev {
                cur_run = 1;
                cur_start = date;
            } else if date == prev + chrono::Duration::days(1) {
                cur_run += 1;
            } else {
                if cur_run > longest_streak_days {
                    longest_streak_days = cur_run;
                    longest_streak_range = Some(format!(
                        "{}-{} {}",
                        cur_start.format("%e").to_string().trim(),
                        prev.format("%e").to_string().trim(),
                        prev.format("%b")
                    ));
                }
                cur_run = 1;
                cur_start = date;
            }
            prev = date;
        }
        if cur_run > longest_streak_days {
            longest_streak_days = cur_run;
            longest_streak_range = Some(format!(
                "{}-{} {}",
                cur_start.format("%e").to_string().trim(),
                prev.format("%e").to_string().trim(),
                prev.format("%b")
            ));
        }

        let today = chrono::Utc::now().date_naive();
        let yesterday = today - chrono::Duration::days(1);
        let mut test_date = if active_dates.contains(&today) {
            today
        } else if active_dates.contains(&yesterday) {
            yesterday
        } else {
            today - chrono::Duration::days(2)
        };

        while active_dates.contains(&test_date) {
            current_streak_days += 1;
            test_date -= chrono::Duration::days(1);
        }
    }

    #[allow(clippy::type_complexity)]
    let longest_task: Option<(i64, String)> = sqlx::query_as(
        "SELECT EXTRACT(EPOCH FROM (pp.finished_at - pp.created_at))::bigint AS duration, \
                p.name AS project_name \
         FROM project_publishes pp \
         JOIN projects p ON p.id = pp.project_id \
         WHERE (pp.created_by = $1 OR p.user_id = $1) \
           AND pp.finished_at IS NOT NULL \
           AND pp.finished_at >= pp.created_at \
         ORDER BY duration DESC \
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let (longest_task_secs, longest_task_name) = match longest_task {
        Some((secs, name)) => (Some(secs), Some(name)),
        None => (None, None),
    };

    let model_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT COALESCE(NULLIF(label, ''), model_id) AS model_name, \
                COALESCE(ROUND(SUM(input_tokens - cached_input_tokens + output_tokens + cached_input_tokens * 0.1)), 0)::bigint AS tokens \
         FROM llm_usage \
         WHERE user_id = $1 AND created_at >= (now() - interval '30 days') \
         GROUP BY model_name \
         ORDER BY tokens DESC \
         LIMIT 5",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let total_30d: i64 = model_rows.iter().map(|(_, t)| *t).sum();
    let top_models: Vec<ProfileModelUsage> = model_rows
        .into_iter()
        .map(|(label, tokens)| {
            let percentage = if total_30d > 0 {
                ((tokens as f64 / total_30d as f64) * 100.0).round()
            } else {
                0.0
            };
            ProfileModelUsage { label, tokens, percentage }
        })
        .collect();

    let crew_interactions: Vec<(String, i64)> = sqlx::query_as(
        "SELECT COALESCE(ad.target_agent_type, 'chitchat') AS agent_type, \
                COUNT(DISTINCT ad.session_id)::bigint AS cnt \
         FROM agent_delegations ad \
         JOIN session_participants sp ON sp.session_id = ad.session_id \
         WHERE sp.user_id = $1 \
         GROUP BY agent_type",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let projects_count: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM projects WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let chat_count: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT session_id)::bigint FROM session_participants WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let get_crew_count = |target: &str| -> i64 {
        crew_interactions.iter().find(|(t, _)| t == target).map(|(_, c)| *c).unwrap_or(0)
    };

    let crew_usage = vec![
        ProfileCrewUsage { agent_id: "nomi".into(), name: "Nomi".into(), count: chat_count, kind: "chats".into() },
        ProfileCrewUsage { agent_id: "dana".into(), name: "Dana".into(), count: get_crew_count("money"), kind: "chats".into() },
        ProfileCrewUsage { agent_id: "kala".into(), name: "Kala".into(), count: get_crew_count("reminders"), kind: "chats".into() },
        ProfileCrewUsage { agent_id: "koda".into(), name: "Koda".into(), count: projects_count, kind: "projects".into() },
    ];

    #[allow(clippy::type_complexity)]
    let showcase_rows: Vec<(Uuid, String, Option<String>, String, Option<String>, Option<i64>)> = if !showcase_project_ids.is_empty() {
        sqlx::query_as(
            "SELECT p.id, p.name, p.description, p.status, \
                    (SELECT url FROM project_publishes pp WHERE pp.project_id = p.id AND pp.status = 'succeeded' ORDER BY pp.created_at DESC LIMIT 1) as deploy_url, \
                    (SELECT EXTRACT(EPOCH FROM (finished_at - created_at))::bigint FROM project_publishes pp WHERE pp.project_id = p.id AND pp.status = 'succeeded' AND finished_at IS NOT NULL ORDER BY pp.created_at DESC LIMIT 1) as build_duration_secs \
             FROM projects p \
             WHERE p.id = ANY($1) AND p.user_id = $2 \
             LIMIT 3",
        )
        .bind(&showcase_project_ids)
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as(
            "SELECT p.id, p.name, p.description, p.status, \
                    (SELECT url FROM project_publishes pp WHERE pp.project_id = p.id AND pp.status = 'succeeded' ORDER BY pp.created_at DESC LIMIT 1) as deploy_url, \
                    (SELECT EXTRACT(EPOCH FROM (finished_at - created_at))::bigint FROM project_publishes pp WHERE pp.project_id = p.id AND pp.status = 'succeeded' AND finished_at IS NOT NULL ORDER BY pp.created_at DESC LIMIT 1) as build_duration_secs \
             FROM projects p \
             WHERE p.user_id = $1 \
             ORDER BY p.updated_at DESC \
             LIMIT 3",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    };

    let showcase_projects: Vec<ShowcaseProject> = showcase_rows
        .into_iter()
        .map(|(id, name, description, status, deploy_url, build_duration_secs)| ShowcaseProject {
            id,
            name,
            description,
            status,
            deploy_url,
            build_duration_secs,
        })
        .collect();

    Ok(ProfileResponse {
        display_name,
        username,
        email,
        avatar_url,
        location,
        joined_at,
        plan_name,
        show_stats_publicly,
        show_crew_publicly,
        showcase_project_ids,
        showcase_projects,
        stats: ProfileStats {
            lifetime_tokens,
            peak_day_tokens,
            peak_day_date,
            longest_task_secs,
            longest_task_name,
            longest_streak_days,
            longest_streak_range,
            current_streak_days,
            best_streak_target: longest_streak_days.max(1),
        },
        activity,
        top_models,
        crew_usage,
    })
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_profile(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<ProfileResponse>, (StatusCode, &'static str)> {
    let profile = load_profile_data(&state.pool, claims.sub).await?;
    Ok(Json(profile))
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub show_stats_publicly: Option<bool>,
    pub show_crew_publicly: Option<bool>,
    pub showcase_project_ids: Option<Vec<Uuid>>,
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
        "INSERT INTO user_profiles (user_id, display_name, username, avatar_url, location, show_stats_publicly, show_crew_publicly, showcase_project_ids) \
         VALUES ($1, $2, $3, $4, $5, COALESCE($6, true), COALESCE($7, false), COALESCE($8, '{}'::uuid[])) \
         ON CONFLICT (user_id) DO UPDATE SET \
             display_name = COALESCE($2, user_profiles.display_name), \
             username = COALESCE($3, user_profiles.username), \
             avatar_url = CASE WHEN $4 = '' THEN NULL WHEN $4 IS NOT NULL THEN $4 ELSE user_profiles.avatar_url END, \
             location = COALESCE($5, user_profiles.location), \
             show_stats_publicly = COALESCE($6, user_profiles.show_stats_publicly), \
             show_crew_publicly = COALESCE($7, user_profiles.show_crew_publicly), \
             showcase_project_ids = COALESCE($8, user_profiles.showcase_project_ids), \
             updated_at = now()",
    )
    .bind(claims.sub)
    .bind(req.display_name.as_deref().filter(|s| !s.is_empty()))
    .bind(req.username.as_deref().filter(|s| !s.is_empty()))
    .bind(req.avatar_url.as_deref())
    .bind(req.location.as_deref())
    .bind(req.show_stats_publicly)
    .bind(req.show_crew_publicly)
    .bind(&req.showcase_project_ids)
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

    let profile = load_profile_data(&state.pool, claims.sub).await.map_err(|(status, msg)| (status, msg.to_string()))?;
    Ok(Json(profile))
}

#[derive(Deserialize)]
pub struct CheckUsernameQuery {
    pub username: String,
}

#[derive(Serialize)]
pub struct CheckUsernameResponse {
    pub available: bool,
    pub reason: Option<String>,
}

pub async fn check_username(
    State(state): State<AppState>,
    claims: Option<AuthClaims>,
    Query(query): Query<CheckUsernameQuery>,
) -> Result<Json<CheckUsernameResponse>, StatusCode> {
    let username = query.username.trim().trim_start_matches('@');
    if username.is_empty() || username.len() > 32 {
        return Ok(Json(CheckUsernameResponse {
            available: false,
            reason: Some("Username must be between 1 and 32 characters".to_string()),
        }));
    }
    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
        return Ok(Json(CheckUsernameResponse {
            available: false,
            reason: Some("Username may only contain letters, numbers, underscores, and periods".to_string()),
        }));
    }

    let current_user_id = claims.map(|c| c.0.sub).unwrap_or(Uuid::nil());
    let taken: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM user_profiles WHERE LOWER(username) = LOWER($1) AND user_id <> $2)",
    )
    .bind(username)
    .bind(current_user_id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);

    if taken {
        Ok(Json(CheckUsernameResponse {
            available: false,
            reason: Some("Username is already taken".to_string()),
        }))
    } else {
        Ok(Json(CheckUsernameResponse {
            available: true,
            reason: None,
        }))
    }
}

#[derive(Serialize)]
pub struct PublicProfileResponse {
    pub display_name: Option<String>,
    pub username: String,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub plan_name: String,
    pub show_stats_publicly: bool,
    pub show_crew_publicly: bool,
    pub stats: Option<ProfileStats>,
    pub activity: Option<Vec<ProfileDayActivity>>,
    pub top_models: Option<Vec<ProfileModelUsage>>,
    pub crew_usage: Option<Vec<ProfileCrewUsage>>,
    pub showcase_projects: Vec<ShowcaseProject>,
}

pub async fn get_public_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<PublicProfileResponse>, (StatusCode, &'static str)> {
    let clean_username = username.trim().trim_start_matches('@');
    let user_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM user_profiles WHERE LOWER(username) = LOWER($1)",
    )
    .bind(clean_username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to query public profile by username");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load profile")
    })?;

    let Some(user_id) = user_id else {
        return Err((StatusCode::NOT_FOUND, "profile not found"));
    };

    let profile = load_profile_data(&state.pool, user_id).await?;

    Ok(Json(PublicProfileResponse {
        display_name: profile.display_name,
        username: profile.username.unwrap_or_else(|| clean_username.to_string()),
        avatar_url: profile.avatar_url,
        location: profile.location,
        joined_at: profile.joined_at,
        plan_name: profile.plan_name,
        show_stats_publicly: profile.show_stats_publicly,
        show_crew_publicly: profile.show_crew_publicly,
        stats: if profile.show_stats_publicly { Some(profile.stats) } else { None },
        activity: if profile.show_stats_publicly { Some(profile.activity) } else { None },
        top_models: if profile.show_stats_publicly { Some(profile.top_models) } else { None },
        crew_usage: if profile.show_crew_publicly { Some(profile.crew_usage) } else { None },
        showcase_projects: profile.showcase_projects,
    }))
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

/// Largest profile picture accepted.
pub const MAX_AVATAR_BYTES: usize = 5 * 1024 * 1024;

#[derive(Serialize)]
pub struct AvatarUploadResponse {
    pub avatar_url: String,
}

/// Whether `bytes` really is the image `content_type` says, by its first bytes.
fn looks_like(content_type: &str, bytes: &[u8]) -> bool {
    match content_type {
        "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "image/webp" => bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        _ => false,
    }
}

/// `POST /api/profile/avatar`: the picture itself as the body. Nomi keeps it where chat files go
/// (the bucket, or disk without one) and serves it itself at `/avatars/<user>/<file>`, so it works
/// whether or not the bucket is public, and without one at all.
#[tracing::instrument(skip(state, claims, headers, body))]
pub async fn upload_avatar(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<AvatarUploadResponse>, (StatusCode, String)> {
    let content_type = headers.get(axum::http::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let extension = match content_type.as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => return Err((StatusCode::BAD_REQUEST, "the picture must be a PNG, JPEG, WebP or GIF".to_string())),
    };
    if body.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "the picture is empty".to_string()));
    }
    if body.len() > MAX_AVATAR_BYTES {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, format!("the picture can be at most {} MB", MAX_AVATAR_BYTES / 1024 / 1024)));
    }
    if !looks_like(&content_type, &body) {
        return Err((StatusCode::BAD_REQUEST, "that file isn't the image it says it is".to_string()));
    }

    let file = format!("{}.{extension}", Uuid::new_v4());
    let key = avatar_key(claims.sub, &file);
    nomi_storage::blob::attachment_store().put_bytes(&key, body.to_vec(), &content_type).await.map_err(|e| {
        tracing::error!(error = %e, "failed to store avatar");
        (StatusCode::BAD_GATEWAY, "couldn't store the picture".to_string())
    })?;
    let avatar_url = format!("/avatars/{}/{file}", claims.sub);
    sqlx::query(
        "INSERT INTO user_profiles (user_id, avatar_url) VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE SET avatar_url = $2, updated_at = now()",
    )
    .bind(claims.sub)
    .bind(&avatar_url)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to save avatar url");
        (StatusCode::INTERNAL_SERVER_ERROR, "couldn't save the picture".to_string())
    })?;
    Ok(Json(AvatarUploadResponse { avatar_url }))
}

fn avatar_key(user_id: Uuid, file: &str) -> String {
    format!("avatars/{user_id}/{file}")
}

/// The content type of an avatar file name Nomi made (`<uuid>.<ext>`), or None for anything else.
fn avatar_content_type(file: &str) -> Option<&'static str> {
    let (stem, extension) = file.rsplit_once('.')?;
    stem.parse::<Uuid>().ok()?;
    match extension {
        "png" => Some("image/png"),
        "jpg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        _ => None,
    }
}

/// `GET /api/avatars/:user_id/:file`: a profile picture.
pub async fn get_avatar(
    Path((user_id, file)): Path<(Uuid, String)>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    use axum::response::IntoResponse;
    let content_type = avatar_content_type(&file).ok_or((StatusCode::NOT_FOUND, "not found".to_string()))?;
    let bytes = nomi_storage::blob::attachment_store()
        .get_bytes(&avatar_key(user_id, &file))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to read avatar");
            (StatusCode::BAD_GATEWAY, "couldn't read the picture".to_string())
        })?
        .ok_or((StatusCode::NOT_FOUND, "not found".to_string()))?;
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, content_type),
            // A new picture gets a new file name, so a stored one never changes.
            (axum::http::header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            (axum::http::header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    )
        .into_response())
}

#[cfg(test)]
mod avatar_tests {
    use super::avatar_content_type;

    #[test]
    fn only_serves_file_names_nomi_made() {
        assert_eq!(avatar_content_type("6f1c3f0e-6b8a-4d3c-9a77-2f1d3c4b5a69.png"), Some("image/png"));
        assert_eq!(avatar_content_type("../secret.png"), None);
        assert_eq!(avatar_content_type("6f1c3f0e-6b8a-4d3c-9a77-2f1d3c4b5a69.svg"), None);
    }
}

#[derive(Serialize)]
pub struct PreferencesResponse {
    pub theme: String,
    pub accent_color: String,
    pub timezone: String,
    /// "en" or "id": the app's language and the language the crew replies in.
    pub language: String,
    // Distinguishes "the user explicitly saved a timezone (even 'UTC')" from "no
    // user_preferences row exists yet, so this is just the fallback default" — the frontend's
    // browser-detected-timezone auto-save only fires in the latter case.
    pub has_stored_timezone: bool,
    /// Features pinned to the navigation drawer, in order; null until the person customises it.
    pub drawer_pins: Option<Vec<String>>,
    /// The home page layout the person picked; null means the default one.
    pub home_layout: Option<String>,
    /// The animated crew background behind chats and the Stage home.
    pub animated_backdrop: bool,
    /// Recent chats listed in the navigation drawer, and how many.
    pub drawer_recents: bool,
    pub drawer_recents_limit: i16,
}

/// How many recent chats the drawer lists until the person picks another of the allowed sizes.
const DEFAULT_DRAWER_RECENTS_LIMIT: i16 = 10;
const DRAWER_RECENTS_LIMITS: [i16; 4] = [5, 10, 15, 20];

#[tracing::instrument(skip(state, claims))]
pub async fn get_preferences(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<PreferencesResponse>, (StatusCode, &'static str)> {
    #[allow(clippy::type_complexity)]
    let row: Option<(Option<String>, Option<String>, Option<String>, Option<String>, Option<Vec<String>>, Option<String>, Option<bool>, Option<bool>, Option<i16>)> =
        sqlx::query_as(
            "SELECT theme, accent_color, timezone, language, drawer_pins, home_layout, animated_backdrop, drawer_recents, drawer_recents_limit FROM user_preferences WHERE user_id = $1",
        )
            .bind(claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to load preferences");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to load preferences")
            })?;

    let (theme, accent_color, timezone, language, drawer_pins, home_layout, animated_backdrop, drawer_recents, drawer_recents_limit) =
        row.unwrap_or((None, None, None, None, None, None, None, None, None));
    Ok(Json(PreferencesResponse {
        theme: theme.unwrap_or_else(|| "system".to_string()),
        accent_color: accent_color.unwrap_or_else(|| DEFAULT_ACCENT_COLOR.to_string()),
        has_stored_timezone: timezone.is_some(),
        timezone: timezone.unwrap_or_else(|| "UTC".to_string()),
        language: Locale::from_code_or_default(language.as_deref()).code().to_string(),
        drawer_pins,
        home_layout,
        animated_backdrop: animated_backdrop.unwrap_or(false),
        drawer_recents: drawer_recents.unwrap_or(true),
        drawer_recents_limit: drawer_recents_limit.unwrap_or(DEFAULT_DRAWER_RECENTS_LIMIT),
    }))
}

/// All three fields are optional so the Preferences page's separate Theme, Accent color, and
/// Timezone forms can each update just their own field via one shared endpoint, without
/// clobbering the others — see put_preferences' update-then-insert-if-missing below.
#[derive(Deserialize)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<String>,
    pub accent_color: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub drawer_pins: Option<Vec<String>>,
    pub home_layout: Option<String>,
    pub animated_backdrop: Option<bool>,
    pub drawer_recents: Option<bool>,
    pub drawer_recents_limit: Option<i16>,
}

/// A home layout is a slug the frontend knows (its layouts live in $lib/homeLayouts), so adding
/// one needs no backend change; the frontend falls back to its default for a slug it doesn't know.
fn is_home_layout_slug(value: &str) -> bool {
    (1..=32).contains(&value.len())
        && value.starts_with(|c: char| c.is_ascii_lowercase())
        && value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Features that can be pinned to the drawer (Home, Chats and Projects are always there).
const PINNABLE_FEATURES: [&str; 8] = ["money", "reminders", "memory", "notifications", "crew", "connections", "models", "billing"];

const ALLOWED_THEMES: [&str; 3] = ["light", "dark", "system"];
/// The theme people start on until they pick one (the column default says the same).
const DEFAULT_ACCENT_COLOR: &str = "lagoon";

/// Appearance themes (Preferences → Appearance); the column keeps its accent_color name.
const ALLOWED_ACCENT_COLORS: [&str; 12] = ["canopy", "coral-reef", "borneo-dusk", "phantom", "senja-jakarta", "lautan", "matcha", "saffron", "lagoon", "orchid", "ruby", "graphite"];

#[tracing::instrument(skip(state, claims, req))]
pub async fn put_preferences(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<UpdatePreferencesRequest>,
) -> Result<Json<PreferencesResponse>, (StatusCode, &'static str)> {
    if let Some(theme) = &req.theme {
        if !ALLOWED_THEMES.contains(&theme.as_str()) {
            return Err((StatusCode::BAD_REQUEST, "theme must be 'light', 'dark', or 'system'"));
        }
    }
    if let Some(accent_color) = &req.accent_color {
        if !ALLOWED_ACCENT_COLORS.contains(&accent_color.as_str()) {
            return Err((StatusCode::BAD_REQUEST, "accent_color must be one of the appearance themes"));
        }
    }
    if let Some(language) = &req.language {
        if Locale::from_code(language).is_none() {
            return Err((StatusCode::BAD_REQUEST, "language must be 'en' or 'id'"));
        }
    }
    let language = req.language.as_deref().and_then(Locale::from_code).map(Locale::code);
    if let Some(timezone) = &req.timezone {
        if timezone.parse::<chrono_tz::Tz>().is_err() {
            return Err((StatusCode::BAD_REQUEST, "timezone must be a valid IANA timezone name"));
        }
    }
    if let Some(pins) = &req.drawer_pins {
        if pins.iter().any(|pin| !PINNABLE_FEATURES.contains(&pin.as_str())) {
            return Err((StatusCode::BAD_REQUEST, "drawer_pins may only name pinnable features"));
        }
        if pins.iter().enumerate().any(|(i, pin)| pins[..i].contains(pin)) {
            return Err((StatusCode::BAD_REQUEST, "drawer_pins can't list a feature twice"));
        }
    }
    if let Some(limit) = req.drawer_recents_limit {
        if !DRAWER_RECENTS_LIMITS.contains(&limit) {
            return Err((StatusCode::BAD_REQUEST, "drawer_recents_limit must be 5, 10, 15 or 20"));
        }
    }
    if let Some(layout) = &req.home_layout {
        if !is_home_layout_slug(layout) {
            return Err((StatusCode::BAD_REQUEST, "home_layout must be a layout's slug"));
        }
    }

    let mut tx = state.pool.begin().await.map_err(|e| {
        tracing::error!(error = %e, "failed to save preferences");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences")
    })?;

    #[allow(clippy::type_complexity)]
    let updated: Option<(String, String, String, String, Option<Vec<String>>, Option<String>, Option<bool>, Option<bool>, Option<i16>)> = sqlx::query_as(
        "UPDATE user_preferences SET \
            theme = COALESCE($2, theme), \
            accent_color = COALESCE($3, accent_color), \
            timezone = COALESCE($4, timezone), \
            language = COALESCE($5, language), \
            drawer_pins = COALESCE($6, drawer_pins), \
            home_layout = COALESCE($7, home_layout), \
            animated_backdrop = COALESCE($8, animated_backdrop), \
            drawer_recents = COALESCE($9, drawer_recents), \
            drawer_recents_limit = COALESCE($10, drawer_recents_limit), \
            updated_at = now() \
         WHERE user_id = $1 \
         RETURNING theme, accent_color, timezone, language, drawer_pins, home_layout, animated_backdrop, drawer_recents, drawer_recents_limit",
    )
    .bind(claims.sub)
    .bind(&req.theme)
    .bind(&req.accent_color)
    .bind(&req.timezone)
    .bind(language)
    .bind(&req.drawer_pins)
    .bind(&req.home_layout)
    .bind(req.animated_backdrop)
    .bind(req.drawer_recents)
    .bind(req.drawer_recents_limit)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to save preferences");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences")
    })?;

    let (theme, accent_color, timezone, language, drawer_pins, home_layout, animated_backdrop, drawer_recents, drawer_recents_limit) = match updated {
        Some(row) => row,
        None => sqlx::query_as(
            "INSERT INTO user_preferences (user_id, theme, accent_color, timezone, language, drawer_pins, home_layout, animated_backdrop, drawer_recents, drawer_recents_limit) \
             VALUES ($1, COALESCE($2, 'system'), COALESCE($3, 'lagoon'), COALESCE($4, 'UTC'), COALESCE($5, 'en'), $6, $7, $8, $9, $10) \
             RETURNING theme, accent_color, timezone, language, drawer_pins, home_layout, animated_backdrop, drawer_recents, drawer_recents_limit",
        )
        .bind(claims.sub)
        .bind(&req.theme)
        .bind(&req.accent_color)
        .bind(&req.timezone)
        .bind(language)
        .bind(&req.drawer_pins)
        .bind(&req.home_layout)
        .bind(req.animated_backdrop)
        .bind(req.drawer_recents)
        .bind(req.drawer_recents_limit)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to save preferences");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences")
        })?,
    };

    tx.commit().await.map_err(|e| {
        tracing::error!(error = %e, "failed to save preferences");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences")
    })?;

    Ok(Json(PreferencesResponse {
        theme,
        accent_color,
        timezone,
        language,
        has_stored_timezone: true,
        drawer_pins,
        home_layout,
        animated_backdrop: animated_backdrop.unwrap_or(false),
        drawer_recents: drawer_recents.unwrap_or(true),
        drawer_recents_limit: drawer_recents_limit.unwrap_or(DEFAULT_DRAWER_RECENTS_LIMIT),
    }))
}
