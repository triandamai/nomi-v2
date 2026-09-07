use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PermissionDecision {
    Allow,
    Deny,
    Ask,
}

/// `*`-only glob match (no `?`, no character classes) — enough for path prefixes like `src/*`
/// or `*.env`. Matched in application code, not SQL: Postgres has no glob operator.
fn glob_match(pattern: &str, text: &str) -> bool {
    fn helper(p: &[u8], t: &[u8]) -> bool {
        match (p.first(), t.first()) {
            (None, None) => true,
            (Some(b'*'), _) => helper(&p[1..], t) || (!t.is_empty() && helper(p, &t[1..])),
            (Some(pc), Some(tc)) if pc == tc => helper(&p[1..], &t[1..]),
            _ => false,
        }
    }
    helper(pattern.as_bytes(), text.as_bytes())
}

/// Looks up `tool_permission_rules` for `(user_id, tool_name)`. A rule with a `path_pattern`
/// only applies when the tool call's `input.path` matches it; a rule with `path_pattern = NULL`
/// matches any call to that tool. A pattern match beats a wildcard match when both exist. No
/// matching rule at all → `Ask`.
pub async fn check_tool_permission(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    tool_name: &str,
    input: &Value,
) -> PermissionDecision {
    let path = input.get("path").and_then(|v| v.as_str());

    let rules: Vec<(Option<String>, String)> = sqlx::query_as(
        "SELECT path_pattern, decision FROM tool_permission_rules WHERE user_id = $1 AND tool_name = $2",
    )
    .bind(user_id)
    .bind(tool_name)
    .fetch_all(&mut **conn)
    .await
    .unwrap_or_default();

    let mut pattern_match: Option<String> = None;
    let mut wildcard_match: Option<String> = None;

    for (pattern, decision) in rules {
        match (&pattern, path) {
            (Some(p), Some(path)) if glob_match(p, path) => pattern_match = Some(decision),
            (None, _) => wildcard_match = Some(decision),
            _ => {}
        }
    }

    match pattern_match.or(wildcard_match).as_deref() {
        Some("allow") => PermissionDecision::Allow,
        Some("deny") => PermissionDecision::Deny,
        _ => PermissionDecision::Ask,
    }
}

/// Persists an "always allow/deny" decision from an approval card so future calls to this same
/// tool (optionally scoped to a path pattern) skip the approval prompt.
pub async fn remember_decision(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    tool_name: &str,
    path_pattern: Option<&str>,
    decision: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, path_pattern, decision) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(tool_name)
        .bind(path_pattern)
        .bind(decision)
        .execute(&mut **conn)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_star_matches_any_suffix() {
        assert!(glob_match("src/*", "src/index.html"));
        assert!(glob_match("*.env", "backend/.env"));
        assert!(!glob_match("src/*", "lib/index.html"));
    }

    #[test]
    fn glob_exact_match_with_no_star() {
        assert!(glob_match("index.html", "index.html"));
        assert!(!glob_match("index.html", "index.htm"));
    }
}
