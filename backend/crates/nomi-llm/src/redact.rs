//! Strips secrets out of text before it is stored, logged, shown to a person or handed to a
//! model: API keys and tokens in URLs, headers and JSON, well-known key formats, and passwords
//! in connection strings. Errors from HTTP clients quote the request URL, and providers echo
//! request details back, so every error message passes through here.

use std::borrow::Cow;
use std::sync::OnceLock;

use regex::Regex;

/// What a secret is replaced with.
pub const MASK: &str = "[REDACTED]";

struct Rule {
    pattern: Regex,
    /// The replacement, with `$pre` for the part kept before the secret.
    replacement: &'static str,
}

fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        let rule = |pattern: &str, replacement: &'static str| Rule { pattern: Regex::new(pattern).expect("redaction pattern"), replacement };
        vec![
            // Query parameters and form fields: ?key=…, &api_key=…, access_token=…, client_secret=…
            rule(
                r#"(?i)(?P<pre>[?&;\s,]?(?:api[_-]?key|key|token|access[_-]?token|refresh[_-]?token|id[_-]?token|auth|secret|client[_-]?secret|password|passwd|pwd|signature|sig|x-amz-signature|x-goog-signature)=)[^&\s)\]'"<>,;]+"#,
                "$pre[REDACTED]",
            ),
            // JSON and YAML values: "api_key": "…", apiKey: '…'
            rule(
                r#"(?i)(?P<pre>["']?(?:api[_-]?key|apikey|x-api-key|x-goog-api-key|token|access[_-]?token|refresh[_-]?token|secret|client[_-]?secret|password|authorization)["']?\s*[:=]\s*["'])[^"']+"#,
                "$pre[REDACTED]",
            ),
            // Authorization headers and bearer tokens.
            rule(r"(?i)(?P<pre>\bbearer\s+)[A-Za-z0-9._~+/=-]{12,}", "$pre[REDACTED]"),
            rule(r"(?P<pre>\bBasic\s+)[A-Za-z0-9+/]{16,}={0,2}", "$pre[REDACTED]"),
            rule(
                r"(?i)(?P<pre>\b(?:x-api-key|x-goog-api-key|api-key|authorization)\s*:\s*)[^\s,;]+",
                "$pre[REDACTED]",
            ),
            // Passwords in connection strings: scheme://user:password@host
            rule(r"(?P<pre>\b[a-zA-Z][a-zA-Z0-9+.-]*://[^\s:/@]+:)[^\s@/]+(?P<at>@)", "$pre[REDACTED]$at"),
            // Well-known key formats, wherever they appear.
            rule(r"\bsk-ant-[A-Za-z0-9_-]{16,}", MASK),
            rule(r"\bsk-(?:proj-|or-v1-)?[A-Za-z0-9_-]{20,}", MASK),
            rule(r"\bAIza[0-9A-Za-z_-]{30,}", MASK),
            rule(r"\bAQ\.[0-9A-Za-z_-]{20,}", MASK),
            rule(r"\bya29\.[0-9A-Za-z_-]{20,}", MASK),
            rule(r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{30,}", MASK),
            rule(r"\bgithub_pat_[A-Za-z0-9_]{30,}", MASK),
            rule(r"\bxox[abposr]-[A-Za-z0-9-]{10,}", MASK),
            rule(r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b", MASK),
            rule(r"\b(?:ship_int|whsec|tvly|sk_live|sk_test|rk_live|pk_live)_[A-Za-z0-9-]{12,}", MASK),
            rule(r"\bhf_[A-Za-z0-9]{30,}", MASK),
            // JSON web tokens.
            rule(r"\beyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}", MASK),
            // Private keys.
            rule(r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----", MASK),
        ]
    })
}

/// `text` with every secret it contains replaced by [`MASK`]. Borrowed when nothing matched.
pub fn redact(text: &str) -> Cow<'_, str> {
    let mut out: Cow<'_, str> = Cow::Borrowed(text);
    for rule in rules() {
        if rule.pattern.is_match(&out) {
            out = Cow::Owned(rule.pattern.replace_all(&out, rule.replacement).into_owned());
        }
    }
    out
}

/// [`redact`] as an owned string.
pub fn redacted(text: impl AsRef<str>) -> String {
    redact(text.as_ref()).into_owned()
}

#[cfg(test)]
mod tests {
    use super::redact;

    fn clean(text: &str) -> String {
        redact(text).into_owned()
    }

    #[test]
    fn query_parameters() {
        let raw = "error sending request for url (https://generativelanguage.googleapis.com/v1beta/models/x:streamGenerateContent?alt=sse&key=AQ.Ab8xxsecretxx)";
        let out = clean(raw);
        assert!(!out.contains("Ab8xxsecretxx"), "{out}");
        assert!(out.contains("alt=sse&key=[REDACTED])"), "{out}");
        assert_eq!(clean("https://e.com/api?token=sec123&api_key=sec456&safe=true"), "https://e.com/api?token=[REDACTED]&api_key=[REDACTED]&safe=true");
        assert_eq!(clean("POST /oauth client_secret=abc123def&grant=x"), "POST /oauth client_secret=[REDACTED]&grant=x");
    }

    #[test]
    fn headers_and_json() {
        let header = clean("Authorization: Bearer abcdefghijklmnop123");
        assert!(header.starts_with("Authorization: [REDACTED]") && !header.contains("abcdefghijklmnop123"), "{header}");
        assert_eq!(clean("sent bearer abcdefghijklmnop123 to it"), "sent bearer [REDACTED] to it");
        assert_eq!(clean(r#"{"api_key": "hunter2-secret", "model": "x"}"#), r#"{"api_key": "[REDACTED]", "model": "x"}"#);
        assert_eq!(clean("x-goog-api-key: AIzaSyD-not-real"), "x-goog-api-key: [REDACTED]");
    }

    #[test]
    fn known_formats_and_connection_strings() {
        for secret in [
            "sk-ant-api03-abcdefghijklmnopqrstuvwxyz",
            "sk-proj-abcdefghijklmnopqrstuvwxyz012345",
            "sk-or-v1-abcdefghijklmnopqrstuvwxyz0123",
            "AIzaSyA1234567890abcdefghijklmnopqrstu",
            "ghp_abcdefghijklmnopqrstuvwxyz0123456789",
            "github_pat_11ABCDEFG0123456789_abcdefghijklmnop",
            "xoxb-1234567890-abcdefghij",
            "AKIAABCDEFGHIJKLMNOP",
            "ship_int_0123456789abcdef0123",
            "whsec_0123456789abcdef",
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTYifQ.abcdefghijklmnop",
        ] {
            let out = clean(&format!("failed with {secret} today"));
            assert!(!out.contains(secret), "{secret} leaked: {out}");
            assert!(out.contains("[REDACTED]"));
        }
        assert_eq!(clean("postgres://app:s3cr3t@db.internal:5432/app"), "postgres://app:[REDACTED]@db.internal:5432/app");
    }

    #[test]
    fn ordinary_text_is_untouched() {
        for text in [
            "The model returned 429: rate limited, retry after 20s",
            "https://example.com/docs?page=2&sort=asc",
            "key points: keep it short",
            "token budget exceeded",
            "a basic understanding of authorization flows",
        ] {
            assert_eq!(clean(text), text);
        }
    }
}
