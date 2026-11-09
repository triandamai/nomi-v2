//! Scrubbing secrets out of text that may end up in logs, error messages or the database.
//!
//! A sandbox's preview address carries its access token in the query string (`?_token=...`), a
//! signed upload address carries its signature there, and a provider error body may echo either
//! of them (or the API key) back. Anything built from provider text goes through here first.

/// What a removed secret is replaced with.
pub const REDACTED: &str = "[redacted]";
/// The query parameter boat puts a private preview's access token in.
pub const TOKEN_PARAM: &str = "_token=";
/// Separates a URL's scheme from the rest.
const SCHEME_SEPARATOR: &str = "://";

/// True for a character that cannot be part of a URL as it appears in prose, JSON or XML.
fn ends_url(c: char) -> bool {
    c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | '`')
}

/// True for a character allowed in a URL scheme.
fn scheme_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')
}

/// Strip what can carry a secret from one URL: its user info, query string and fragment.
fn scrub_url(url: &str) -> String {
    let Some(sep) = url.find(SCHEME_SEPARATOR) else { return url.to_string() };
    let (scheme, rest) = url.split_at(sep + SCHEME_SEPARATOR.len());
    let cut = rest.find(['?', '#']).unwrap_or(rest.len());
    let (head, tail) = rest.split_at(cut);
    let authority_end = head.find('/').unwrap_or(head.len());
    let head = match head[..authority_end].rfind('@') {
        Some(at) => format!("{REDACTED}@{}", &head[at + 1..]),
        None => head.to_string(),
    };
    if tail.is_empty() {
        format!("{scheme}{head}")
    } else {
        format!("{scheme}{head}?{REDACTED}")
    }
}

/// Remove URL user info, query strings and fragments, and any stray `_token=` value, from `text`.
pub fn redact_urls(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(sep) = rest.find(SCHEME_SEPARATOR) {
        let start = rest[..sep].char_indices().rev().take_while(|(_, c)| scheme_char(*c)).last().map_or(sep, |(i, _)| i);
        let end = rest[sep..].find(ends_url).map_or(rest.len(), |e| sep + e);
        out.push_str(&rest[..start]);
        out.push_str(&scrub_url(&rest[start..end]));
        rest = &rest[end..];
    }
    out.push_str(rest);
    redact_token_params(&out)
}

/// Replace the value of every `_token=` left in `text` (outside a URL) with `REDACTED`.
fn redact_token_params(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(TOKEN_PARAM) {
        let value_start = at + TOKEN_PARAM.len();
        out.push_str(&rest[..value_start]);
        let value_len = rest[value_start..].find(|c: char| ends_url(c) || c == '&').unwrap_or(rest.len() - value_start);
        out.push_str(REDACTED);
        rest = &rest[value_start + value_len..];
    }
    out.push_str(rest);
    out
}

/// `redact_urls`, and every occurrence of `secret` (when it isn't empty) replaced too.
pub fn redact(text: &str, secret: &str) -> String {
    let text = if secret.is_empty() { text.to_string() } else { text.replace(secret, REDACTED) };
    redact_urls(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_query_strings_and_keeps_the_host() {
        let out = redact_urls("could not reach https://x-3000.on.boat.dev/?_token=s3cret refused");
        assert_eq!(out, "could not reach https://x-3000.on.boat.dev/?[redacted] refused");
    }

    #[test]
    fn strips_urls_inside_json_and_xml() {
        let out = redact_urls(r#"{"url":"https://up.example.com/a?X-Sig=abc&e=1"} <Url>https://s3.test/b?sig=def</Url>"#);
        assert!(!out.contains("abc") && !out.contains("def"), "{out}");
        assert!(out.contains("https://up.example.com/a?[redacted]") && out.contains("https://s3.test/b?[redacted]"), "{out}");
    }

    #[test]
    fn strips_user_info_and_fragments() {
        assert_eq!(redact_urls("postgres://nomi:pw@db:5432/nomi"), "postgres://[redacted]@db:5432/nomi");
        assert_eq!(redact_urls("https://h/p#tok"), "https://h/p?[redacted]");
    }

    #[test]
    fn strips_a_bare_token_parameter() {
        assert_eq!(redact_urls("token was _token=abc&x=1 here"), "token was _token=[redacted]&x=1 here");
    }

    #[test]
    fn leaves_plain_text_alone() {
        assert_eq!(redact_urls("npm ERR! 404 not found"), "npm ERR! 404 not found");
        assert_eq!(redact_urls("see http://localhost:5173/"), "see http://localhost:5173/");
    }

    #[test]
    fn removes_the_secret_everywhere() {
        assert_eq!(redact("bad key boat_k1 (boat_k1)", "boat_k1"), "bad key [redacted] ([redacted])");
        assert_eq!(redact("nothing", ""), "nothing");
    }
}
