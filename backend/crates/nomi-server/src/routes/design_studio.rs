//! The Design Studio's routes. A chat's previews are listed for the people who can open the
//! chat, while Sora is on. A version's files are served to the Studio's frame by the
//! version's unguessable name alone: the frame is sandboxed into an origin of its own, so it
//! sends no sign-in and can reach nothing of Nomi's, and the response says so again in case the
//! address is opened outside the Studio.

use axum::{
    extract::{Path, State},
    http::{header, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use nomi_agent_core::crew::DESIGN_AGENT_TYPE;
use nomi_agent_core::design_approval::{self, Approval};
use nomi_agent_core::design_artifact::{self, Element, Page, ENTRY_PAGE, PREVIEW_TOKEN_CHARS};
use nomi_agent_core::design_directions::{self, Direction};
use nomi_agent_core::design_feedback::{self, Feedback, NewFeedback, MAX_OPEN_PER_CHAT};
use nomi_auth::extractor::AuthClaims;
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::routes::sessions::authorize_session_access;

/// What a preview page may do. `sandbox` without `allow-same-origin` gives it an origin of its
/// own (no cookies, no storage, nothing of Nomi's), and it can't call out, post forms or frame
/// anything. Only a page from the address it is served at (Nomi's own) can put it in a frame.
pub const PREVIEW_POLICY: &str = "sandbox allow-scripts; connect-src 'none'; form-action 'none'; base-uri 'none'; object-src 'none'; frame-src 'none'; frame-ancestors 'self'";
/// A preview's address is a link to pass on by hand, not something to be found by searching.
pub const PREVIEW_ROBOTS: &str = "noindex, nofollow";
/// How long a version's files may be kept by the browser, in seconds: a version never changes.
const PREVIEW_MAX_AGE_SECS: u32 = 3600;
/// The script that lets a page tell the Studio what was pointed at.
const BRIDGE: &str = include_str!("design_bridge.js");
const BODY_END: &str = "</body>";
const HTML_TYPE: &str = "text/html";
/// Who Sora's task comes from when the person approves a version: Nomi, on their behalf.
const APPROVED_BY: &str = "chitchat";

#[derive(Serialize)]
pub struct StudioDirectionSet {
    pub version: i32,
    pub brief_version: Option<i32>,
    pub directions: Vec<Direction>,
}

#[derive(Serialize)]
pub struct StudioPreview {
    pub version: i32,
    pub direction_id: String,
    pub title: String,
    pub pages: Vec<Page>,
    pub elements: Vec<Element>,
    pub preview_token: String,
    pub published_url: Option<String>,
    /// Why it isn't published, when its newest publish failed.
    pub publish_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct StudioResponse {
    /// The newest proposed direction set for the chat, when Sora has proposed directions.
    pub directions: Option<StudioDirectionSet>,
    pub previews: Vec<StudioPreview>,
    /// Everything said about the chat's previews, oldest first.
    pub feedback: Vec<Feedback>,
    /// The approval that stands, when the person approved a version.
    pub approval: Option<Approval>,
}

/// `GET /api/sessions/:id/design`: the chat's preview versions, oldest first.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(session_id): Path<Uuid>) -> Result<Json<StudioResponse>, (StatusCode, &'static str)> {
    if !nomi_settings::config::agent_on(DESIGN_AGENT_TYPE).await {
        return Err((StatusCode::NOT_FOUND, "not found"));
    }
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let failed = |e: sqlx::Error| {
        tracing::error!(error = %e, "design studio: couldn't list previews");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load previews")
    };
    let mut conn = state.pool.acquire().await.map_err(failed)?;
    let directions = design_directions::latest(&mut conn, session_id)
        .await
        .map_err(failed)?
        .map(|(version, brief_version, set)| StudioDirectionSet {
            version,
            brief_version,
            directions: set.directions,
        });
    let previews = design_artifact::list(&mut conn, session_id).await.map_err(failed)?;
    let feedback = design_feedback::list(&mut conn, session_id).await.map_err(failed)?;
    let approval = design_approval::current(&mut conn, session_id).await.map_err(failed)?;
    Ok(Json(StudioResponse {
        directions,
        feedback,
        approval,
        previews: previews
            .into_iter()
            .map(|stored| StudioPreview {
                version: stored.version,
                direction_id: stored.manifest.direction_id,
                title: stored.manifest.title,
                pages: stored.manifest.pages,
                elements: stored.manifest.elements,
                preview_token: stored.preview_token,
                published_url: stored.published_url,
                publish_error: stored.publish_error,
                created_at: stored.created_at,
            })
            .collect(),
    }))
}

type Refused = (StatusCode, String);

fn refused(status: StatusCode, why: &str) -> Refused {
    (status, why.to_string())
}

/// Whether this person may use the Studio for this chat; the refusal otherwise.
async fn studio_access(state: &AppState, user_id: Uuid, session_id: Uuid) -> Result<(), Refused> {
    if !nomi_settings::config::agent_on(DESIGN_AGENT_TYPE).await {
        return Err(refused(StatusCode::NOT_FOUND, "not found"));
    }
    authorize_session_access(&state.pool, user_id, session_id).await.map_err(|(status, why)| refused(status, why))
}

fn unavailable(e: sqlx::Error) -> Refused {
    tracing::error!(error = %e, "design studio: the database couldn't be reached");
    refused(StatusCode::INTERNAL_SERVER_ERROR, "that didn't work, try again")
}

/// The version a request's body names.
fn version_of(input: &serde_json::Value) -> Result<i32, Refused> {
    input
        .get("version")
        .and_then(serde_json::Value::as_i64)
        .and_then(|version| i32::try_from(version).ok())
        .ok_or_else(|| refused(StatusCode::UNPROCESSABLE_ENTITY, "version is required"))
}

/// `POST /api/sessions/:id/design/feedback`: something said about one version, kept for good.
pub async fn add_feedback(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Feedback>), Refused> {
    studio_access(&state, claims.sub, session_id).await?;
    let version = version_of(&input)?;
    let mut conn = state.pool.acquire().await.map_err(unavailable)?;
    let manifest = design_artifact::version(&mut conn, session_id, version).await.map_err(unavailable)?.ok_or_else(|| refused(StatusCode::NOT_FOUND, "no such version"))?;
    let feedback = NewFeedback::parse(version, &manifest, &input).map_err(|why| (StatusCode::UNPROCESSABLE_ENTITY, why))?;
    if design_feedback::open_count(&mut conn, session_id).await.map_err(unavailable)? >= MAX_OPEN_PER_CHAT {
        return Err(refused(StatusCode::UNPROCESSABLE_ENTITY, "there is a lot of feedback waiting already: ask Sora for a new version first"));
    }
    let kept = design_feedback::add(&mut conn, session_id, claims.sub, &feedback).await.map_err(unavailable)?;
    Ok((StatusCode::CREATED, Json(kept)))
}

/// `POST /api/sessions/:id/design/approval`: the person says this version is the design to
/// build. Only a signed-in person who can open the chat can; no agent has a tool for it.
pub async fn approve(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Approval>), Refused> {
    studio_access(&state, claims.sub, session_id).await?;
    let version = version_of(&input)?;
    let note = design_approval::note_of(&input).map_err(|why| (StatusCode::UNPROCESSABLE_ENTITY, why))?;
    let mut conn = state.pool.acquire().await.map_err(unavailable)?;
    if design_artifact::version(&mut conn, session_id, version).await.map_err(unavailable)?.is_none() {
        return Err(refused(StatusCode::NOT_FOUND, "no such version"));
    }
    let approval = design_approval::approve(&mut conn, session_id, claims.sub, version, note.as_deref()).await.map_err(unavailable)?;
    // Approving is the go-ahead to build: Sora writes the handoff and passes it to Koda without
    // the person having to say so in the chat too.
    let task = nomi_agent_core::prompts::DESIGN_APPROVED_TASK.replace("{version}", &version.to_string());
    if let Err(why) = nomi_agent_core::delegation::create_delegation(&mut conn, None, session_id, APPROVED_BY, DESIGN_AGENT_TYPE, &task, claims.sub).await {
        tracing::warn!(error = %why, %session_id, "design studio: the approval didn't reach Sora");
    }
    Ok((StatusCode::CREATED, Json(approval)))
}

/// What publishing a version again came to.
#[derive(Debug, Serialize)]
pub struct PublishOutcome {
    pub version: i32,
    pub published_url: Option<String>,
    pub publish_error: Option<String>,
}

pub fn publish_outcome(version: i32, published: Result<nomi_agent_core::static_publish::Published, String>) -> PublishOutcome {
    match published {
        Ok(at) => PublishOutcome { version, published_url: Some(at.url), publish_error: None },
        Err(why) => PublishOutcome { version, published_url: None, publish_error: Some(why) },
    }
}

/// `POST /api/sessions/:id/design/publish`: publishes a version again, after it couldn't be.
/// The answer says where it is now, or why it still isn't, and both are kept for the Studio.
pub async fn publish(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<PublishOutcome>, Refused> {
    studio_access(&state, claims.sub, session_id).await?;
    let version = version_of(&input)?;
    let (manifest, files) = {
        let mut conn = state.pool.acquire().await.map_err(unavailable)?;
        let manifest = design_artifact::version(&mut conn, session_id, version).await.map_err(unavailable)?.ok_or_else(|| refused(StatusCode::NOT_FOUND, "no such version"))?;
        let listed = design_artifact::list(&mut conn, session_id).await.map_err(unavailable)?;
        let stored = listed.iter().find(|v| v.version == version).ok_or_else(|| refused(StatusCode::NOT_FOUND, "no such version"))?;
        retry_allowed(stored.published_url.as_deref(), stored.publish_error.as_deref())?;
        (manifest, design_artifact::files(&mut conn, session_id, version).await.map_err(unavailable)?)
    };
    // No connection is held through the upload, which can take minutes.
    let request = nomi_agent_core::static_publish::PublishRequest { user_id: claims.sub, session_id, version, title: &manifest.title, files: &files };
    let publisher = crate::static_publisher::ShipyardStaticPublisher::shared(state.pool.clone());
    let published = nomi_agent_core::static_publish::publish(publisher.as_ref(), &request).await;
    let mut conn = state.pool.acquire().await.map_err(unavailable)?;
    let published = nomi_agent_core::static_publish::record_outcome(&mut conn, session_id, version, published).await;
    Ok(Json(publish_outcome(version, published)))
}

/// A version is published again only when it isn't published now: never tried, or its newest
/// publish failed. Each publish uploads every file to Shipyard again.
pub fn retry_allowed(published_url: Option<&str>, publish_error: Option<&str>) -> Result<(), Refused> {
    match (published_url, publish_error) {
        (Some(_), None) => Err(refused(StatusCode::CONFLICT, "this version is already published")),
        _ => Ok(()),
    }
}

/// The file a request path means: nothing is the entry page, a folder its index page.
pub fn file_path(requested: &str) -> String {
    let path = requested.trim_start_matches('/');
    if path.is_empty() {
        ENTRY_PAGE.to_string()
    } else if path.ends_with('/') {
        format!("{path}{ENTRY_PAGE}")
    } else {
        path.to_string()
    }
}

/// `html` with the Studio's script added at the end of its body, told which page it is on.
pub fn with_bridge(page: &str, html: &str) -> String {
    let script = format!("<script>window.__nomiDesignPage={};\n{BRIDGE}</script>", serde_json::Value::String(page.to_string()));
    match html.to_ascii_lowercase().rfind(BODY_END) {
        Some(at) => format!("{}{script}{}", &html[..at], &html[at..]),
        None => format!("{html}{script}"),
    }
}

fn looks_like_a_token(token: &str) -> bool {
    token.len() == PREVIEW_TOKEN_CHARS && token.chars().all(|c| c.is_ascii_hexdigit())
}

async fn serve(state: &AppState, token: &str, requested: &str) -> Response {
    let not_found = || (StatusCode::NOT_FOUND, "not found").into_response();
    if !looks_like_a_token(token) {
        return not_found();
    }
    let path = file_path(requested);
    let file = match state.pool.acquire().await {
        Ok(mut conn) => design_artifact::file_by_token(&mut conn, token, &path).await,
        Err(e) => Err(e),
    };
    let (content_type, content) = match file {
        Ok(Some(file)) => file,
        Ok(None) => return not_found(),
        Err(e) => {
            tracing::error!(error = %e, "design studio: couldn't read a preview file");
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to load the preview").into_response();
        }
    };
    let body = if content_type.starts_with(HTML_TYPE) { with_bridge(&path, &content) } else { content };
    let content_type = HeaderValue::from_str(&content_type).unwrap_or(HeaderValue::from_static("application/octet-stream"));
    let cache = HeaderValue::from_str(&format!("private, max-age={PREVIEW_MAX_AGE_SECS}")).unwrap_or(HeaderValue::from_static("no-store"));
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(PREVIEW_POLICY)),
            (header::REFERRER_POLICY, HeaderValue::from_static("no-referrer")),
            (header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
            (HeaderName::from_static("x-robots-tag"), HeaderValue::from_static(PREVIEW_ROBOTS)),
            // The page's own origin is opaque, so its fonts and module scripts are cross-origin
            // loads; the address is already the whole permission.
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*")),
            (HeaderName::from_static("cross-origin-resource-policy"), HeaderValue::from_static("cross-origin")),
            (header::CACHE_CONTROL, cache),
        ],
        body,
    )
        .into_response()
}

/// `GET /api/design/preview/:token`: the version's entry page.
pub async fn preview_index(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    serve(&state, &token, "").await
}

/// `GET /api/design/preview/:token/*path`: one file of the version.
pub async fn preview_file(State(state): State<AppState>, Path((token, path)): Path<(String, String)>) -> Response {
    serve(&state, &token, &path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_version_that_is_not_published_is_published_again() {
        assert_eq!(retry_allowed(None, Some("publishing previews isn't set up")), Ok(()));
        assert_eq!(retry_allowed(None, None), Ok(()), "never tried");
        assert_eq!(retry_allowed(Some("https://ship.example/dpl/d1"), Some("Shipyard refused it")), Ok(()), "its newest publish failed");
        assert_eq!(retry_allowed(Some("https://ship.example/dpl/d1"), None).unwrap_err().0, StatusCode::CONFLICT);
    }

    #[test]
    fn a_publish_answer_says_where_or_why_not() {
        let at = nomi_agent_core::static_publish::Published { provider: "shipyard".into(), url: "https://ship.example/dpl/d1".into() };
        let done = publish_outcome(3, Ok(at));
        assert_eq!((done.version, done.published_url.as_deref(), done.publish_error), (3, Some("https://ship.example/dpl/d1"), None));
        let failed = publish_outcome(3, Err("publishing previews isn't set up".into()));
        assert_eq!((failed.published_url, failed.publish_error.as_deref()), (None, Some("publishing previews isn't set up")));
    }

    #[test]
    fn a_request_path_means_a_file() {
        assert_eq!(file_path(""), "index.html");
        assert_eq!(file_path("/"), "index.html");
        assert_eq!(file_path("menu/"), "menu/index.html");
        assert_eq!(file_path("menu/index.html"), "menu/index.html");
        assert_eq!(file_path("styles.css"), "styles.css");
    }

    #[test]
    fn the_studios_script_goes_at_the_end_of_the_body_with_the_pages_path() {
        let page = with_bridge("menu/index.html", "<html><body><p>Hi</p></BODY></html>");
        assert!(page.starts_with("<html><body><p>Hi</p><script>window.__nomiDesignPage=\"menu/index.html\";"), "{page}");
        assert!(page.ends_with("</script></BODY></html>"));
        let bare = with_bridge("index.html", "<p>Hi</p>");
        assert!(bare.starts_with("<p>Hi</p><script>") && bare.ends_with("</script>"));
    }

    #[test]
    fn the_script_only_listens_to_the_studio_and_names_itself() {
        assert!(BRIDGE.contains("event.source !== window.parent"));
        assert!(BRIDGE.contains("'nomi-design-preview'") && BRIDGE.contains("'nomi-design-studio'"));
        assert!(BRIDGE.contains("data-design-id"));
        assert!(!BRIDGE.contains("</script"), "it can't end the tag it is put in");
        assert!(BRIDGE.contains("'annotate'") && BRIDGE.contains("point:"), "it marks feedback and says where a click landed");
        assert!(BRIDGE.contains("textContent = String(") && !BRIDGE.contains("innerHTML"), "what the Studio sends is never read as markup");
    }

    #[test]
    fn a_preview_page_gets_an_origin_of_its_own_and_cant_call_out() {
        assert!(PREVIEW_POLICY.starts_with("sandbox allow-scripts;"));
        assert!(!PREVIEW_POLICY.contains("allow-same-origin") && !PREVIEW_POLICY.contains("allow-top-navigation") && !PREVIEW_POLICY.contains("allow-popups"));
        assert!(PREVIEW_POLICY.contains("connect-src 'none'"));
        assert!(PREVIEW_POLICY.contains("frame-ancestors 'self'"), "only Nomi's own pages can frame it");
        assert_eq!(PREVIEW_ROBOTS, "noindex, nofollow", "an address that got out isn't indexed");
    }

    #[test]
    fn only_a_whole_token_is_looked_up() {
        assert!(looks_like_a_token(&"a1".repeat(PREVIEW_TOKEN_CHARS / 2)));
        assert!(!looks_like_a_token("short"));
        assert!(!looks_like_a_token(&"g".repeat(PREVIEW_TOKEN_CHARS)));
    }
}
