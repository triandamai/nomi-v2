//! Nomi's public address (the web app people open), for links that leave the app: emails and
//! the webhook Shipyard calls. `APP_URL` when it's set; otherwise the site of
//! `GOOGLE_REDIRECT_URI`, which already points at the web app on installs with Google sign-in.

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// The scheme and host of `url` (`https://nomi.example.com`), without a trailing slash.
fn origin(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return None;
    }
    Some(parsed.origin().ascii_serialization())
}

fn resolve(app_url: Option<String>, google_redirect: Option<String>) -> Option<String> {
    app_url
        .map(|url| url.trim_end_matches('/').to_string())
        .filter(|url| origin(url).is_some())
        .or_else(|| google_redirect.as_deref().and_then(origin))
}

/// The web app's address, e.g. `https://nomi.example.com` (no trailing slash), or `None` when
/// neither `APP_URL` nor `GOOGLE_REDIRECT_URI` says where it is.
pub fn app_url() -> Option<String> {
    resolve(env("APP_URL"), env("GOOGLE_REDIRECT_URI"))
}

/// `path` (an in-app path like `/invite/abc`) as a full link, or `None` when the address isn't
/// known. Anything that is already a full link is kept as it is.
pub fn link(path: &str) -> Option<String> {
    if origin(path).is_some() {
        return Some(path.to_string());
    }
    let base = app_url()?;
    Some(if path.starts_with('/') { format!("{base}{path}") } else { format!("{base}/{path}") })
}

/// Logged once at startup: without an address, emails can only name the page to open.
pub fn warn_if_unknown() {
    if app_url().is_none() {
        tracing::warn!("APP_URL isn't set: emails can't link to Nomi (set it to the web app's address, e.g. https://nomi.example.com)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_url_comes_first_then_the_google_callbacks_site() {
        let google = Some("https://nomi.example.com/connections/google/callback".to_string());
        assert_eq!(resolve(Some("https://app.example.com/".into()), google.clone()).as_deref(), Some("https://app.example.com"));
        assert_eq!(resolve(None, google.clone()).as_deref(), Some("https://nomi.example.com"));
        assert_eq!(resolve(Some("not a url".into()), google).as_deref(), Some("https://nomi.example.com"));
        assert_eq!(resolve(None, None), None);
        assert_eq!(resolve(Some("https://nomi.example.com/app".into()), None).as_deref(), Some("https://nomi.example.com/app"), "a base path is kept");
    }

    #[test]
    fn origins_are_only_web_addresses() {
        assert_eq!(origin("http://localhost:5173/x").as_deref(), Some("http://localhost:5173"));
        assert_eq!(origin("/invite/abc"), None);
        assert_eq!(origin("javascript:alert(1)"), None);
    }
}
