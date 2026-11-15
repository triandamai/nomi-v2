//! The Design Studio's routes. A chat's previews are listed for the people who can open the
//! chat, where Design is rolled out. A version's files are served to the Studio's frame by the
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
use nomi_agent_core::design_artifact::{self, Element, Page, ENTRY_PAGE, PREVIEW_TOKEN_CHARS};
use nomi_auth::extractor::AuthClaims;
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::routes::sessions::authorize_session_access;

/// What a preview page may do. `sandbox` without `allow-same-origin` gives it an origin of its
/// own (no cookies, no storage, nothing of Nomi's), and it can't call out, post forms or frame
/// anything.
pub const PREVIEW_POLICY: &str = "sandbox allow-scripts; connect-src 'none'; form-action 'none'; base-uri 'none'; object-src 'none'; frame-src 'none'";
/// How long a version's files may be kept by the browser, in seconds: a version never changes.
const PREVIEW_MAX_AGE_SECS: u32 = 3600;
/// The script that lets a page tell the Studio what was pointed at.
const BRIDGE: &str = include_str!("design_bridge.js");
const BODY_END: &str = "</body>";
const HTML_TYPE: &str = "text/html";

#[derive(Serialize)]
pub struct StudioPreview {
    pub version: i32,
    pub direction_id: String,
    pub title: String,
    pub pages: Vec<Page>,
    pub elements: Vec<Element>,
    pub preview_token: String,
    pub published_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct StudioResponse {
    pub previews: Vec<StudioPreview>,
}

/// `GET /api/sessions/:id/design`: the chat's preview versions, oldest first.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(session_id): Path<Uuid>) -> Result<Json<StudioResponse>, (StatusCode, &'static str)> {
    if !nomi_settings::config::design_enabled_for(claims.sub).await {
        return Err((StatusCode::NOT_FOUND, "not found"));
    }
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let failed = |e: sqlx::Error| {
        tracing::error!(error = %e, "design studio: couldn't list previews");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load previews")
    };
    let mut conn = state.pool.acquire().await.map_err(failed)?;
    let previews = design_artifact::list(&mut conn, session_id).await.map_err(failed)?;
    Ok(Json(StudioResponse {
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
                created_at: stored.created_at,
            })
            .collect(),
    }))
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
    }

    #[test]
    fn a_preview_page_gets_an_origin_of_its_own_and_cant_call_out() {
        assert!(PREVIEW_POLICY.starts_with("sandbox allow-scripts;"));
        assert!(!PREVIEW_POLICY.contains("allow-same-origin") && !PREVIEW_POLICY.contains("allow-top-navigation") && !PREVIEW_POLICY.contains("allow-popups"));
        assert!(PREVIEW_POLICY.contains("connect-src 'none'"));
    }

    #[test]
    fn only_a_whole_token_is_looked_up() {
        assert!(looks_like_a_token(&"a1".repeat(PREVIEW_TOKEN_CHARS / 2)));
        assert!(!looks_like_a_token("short"));
        assert!(!looks_like_a_token(&"g".repeat(PREVIEW_TOKEN_CHARS)));
    }
}
