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

/// The scope a remembered rule can be limited to: the call's recipient.
pub const SCOPE_RECIPIENT: &str = "recipient";

/// Keys a tool's input may name who it goes to under.
const RECIPIENT_KEYS: &[&str] = &["to", "recipient", "recipients", "email", "to_email", "address"];

/// Who a call goes to: every string under a recipient key (a string, or a list of them),
/// trimmed and lower-cased. Empty when the call names no one.
pub fn recipients_of(input: &Value) -> Vec<String> {
    let mut found = Vec::new();
    for key in RECIPIENT_KEYS {
        match input.get(*key) {
            Some(Value::String(one)) => found.push(one.trim().to_lowercase()),
            Some(Value::Array(many)) => found.extend(many.iter().filter_map(Value::as_str).map(|s| s.trim().to_lowercase())),
            _ => {}
        }
    }
    found.retain(|r| !r.is_empty());
    found
}

/// Whether a rule limited to `scope` = `value` covers this call: a recipient rule covers a call
/// only when it names someone and every one it names is `value`. An unknown scope covers nothing.
pub fn scope_matches(scope: &str, value: &str, input: &Value) -> bool {
    if scope != SCOPE_RECIPIENT {
        return false;
    }
    let value = value.trim().to_lowercase();
    let recipients = recipients_of(input);
    !recipients.is_empty() && recipients.iter().all(|r| *r == value)
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

    let rules: Vec<(Option<String>, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT path_pattern, decision, scope, scope_value FROM tool_permission_rules WHERE user_id = $1 AND tool_name = $2",
    )
    .bind(user_id)
    .bind(tool_name)
    .fetch_all(&mut **conn)
    .await
    .unwrap_or_default();

    let mut pattern_match: Option<String> = None;
    let mut wildcard_match: Option<String> = None;

    for (pattern, decision, scope, scope_value) in rules {
        // A rule limited to a recipient (or the like) covers only the calls it matches, and
        // beats a rule for the whole tool.
        if let (Some(scope), Some(scope_value)) = (&scope, &scope_value) {
            if scope_matches(scope, scope_value, input) {
                pattern_match = Some(decision);
            }
            continue;
        }
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

/// How long a decision on an approval card covers identical calls in the same chat.
const SESSION_DECISION_WINDOW_MINUTES: i32 = 60;

/// `check_tool_permission`, but first honours what the user already decided on an approval card
/// in this chat for the very same call (same tool, same input) within the last hour. Without
/// this, a model that repeats an approved call, or retries a denied one, asks again every time.
pub async fn check_tool_permission_in_session(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    session_id: Uuid,
    tool_name: &str,
    input: &Value,
) -> PermissionDecision {
    match session_decision(conn, user_id, session_id, tool_name, input).await {
        Some(decision) => decision,
        None => check_tool_permission(conn, user_id, tool_name, input).await,
    }
}

/// What the person decided on an approval card in this chat for the very same call within the
/// last hour, if anything: `Ask` is never returned. A member set to "ask first" (Settings › Crew)
/// honours only this, not the "always" rules.
pub async fn session_decision(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    session_id: Uuid,
    tool_name: &str,
    input: &Value,
) -> Option<PermissionDecision> {
    let recent: Option<String> = sqlx::query_scalar(
        "SELECT decision FROM session_tool_decisions \
         WHERE session_id = $1 AND user_id = $2 AND tool_name = $3 AND input = $4 \
           AND decided_at > now() - make_interval(mins => $5) \
         ORDER BY decided_at DESC LIMIT 1",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(tool_name)
    .bind(input)
    .bind(SESSION_DECISION_WINDOW_MINUTES)
    .fetch_optional(&mut **conn)
    .await
    .ok()
    .flatten();

    match recent.as_deref() {
        Some("allow") => Some(PermissionDecision::Allow),
        Some("deny") => Some(PermissionDecision::Deny),
        _ => None,
    }
}

/// Records the user's decision on an approval card for `check_tool_permission_in_session`.
pub async fn record_session_decision(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    user_id: Uuid,
    tool_name: &str,
    input: &Value,
    approved: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO session_tool_decisions (session_id, user_id, tool_name, input, decision) VALUES ($1, $2, $3, $4, $5)")
        .bind(session_id)
        .bind(user_id)
        .bind(tool_name)
        .bind(input)
        .bind(if approved { "allow" } else { "deny" })
        .execute(&mut **conn)
        .await?;
    Ok(())
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

/// Like `remember_decision`, but the rule covers only calls whose `scope` is `scope_value`
/// (today: the recipient), not the whole tool.
pub async fn remember_scoped_decision(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    tool_name: &str,
    scope: &str,
    scope_value: &str,
    decision: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision, scope, scope_value) VALUES ($1, $2, $3, $4, $5)")
        .bind(user_id)
        .bind(tool_name)
        .bind(decision)
        .bind(scope)
        .bind(scope_value.trim().to_lowercase())
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
    fn recipients_are_read_from_the_usual_keys_lowercased() {
        assert_eq!(recipients_of(&serde_json::json!({"to": " Bu.Sri@Example.com "})), vec!["bu.sri@example.com"]);
        assert_eq!(recipients_of(&serde_json::json!({"recipients": ["a@x.co", "B@x.co"]})), vec!["a@x.co", "b@x.co"]);
        assert!(recipients_of(&serde_json::json!({"subject": "hi"})).is_empty());
    }

    #[test]
    fn a_recipient_rule_covers_only_calls_to_that_recipient() {
        let to = |v: &str| serde_json::json!({"to": v});
        assert!(scope_matches(SCOPE_RECIPIENT, "bu.sri@example.com", &to("Bu.Sri@example.com")));
        assert!(!scope_matches(SCOPE_RECIPIENT, "bu.sri@example.com", &to("boss@example.com")));
        // One of several recipients being the remembered one is not enough.
        assert!(!scope_matches(SCOPE_RECIPIENT, "bu.sri@example.com", &serde_json::json!({"to": ["bu.sri@example.com", "boss@example.com"]})));
        // A call that names no one is never covered, and neither is an unknown scope.
        assert!(!scope_matches(SCOPE_RECIPIENT, "bu.sri@example.com", &serde_json::json!({"subject": "x"})));
        assert!(!scope_matches("channel", "bu.sri@example.com", &to("bu.sri@example.com")));
    }

    #[test]
    fn glob_exact_match_with_no_star() {
        assert!(glob_match("index.html", "index.html"));
        assert!(!glob_match("index.html", "index.htm"));
    }

    async fn pool_user(pool: &sqlx::PgPool) -> Uuid {
        sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn a_recipient_rule_allows_that_recipient_and_still_asks_for_others(pool: sqlx::PgPool) {
        let user_id = pool_user(&pool).await;
        let mut conn = pool.acquire().await.unwrap();
        remember_scoped_decision(&mut conn, user_id, "gmail.send", SCOPE_RECIPIENT, "Bu.Sri@example.com", "allow").await.unwrap();

        let to = |v: &str| serde_json::json!({"to": v});
        assert_eq!(check_tool_permission(&mut conn, user_id, "gmail.send", &to("bu.sri@example.com")).await, PermissionDecision::Allow);
        assert_eq!(check_tool_permission(&mut conn, user_id, "gmail.send", &to("boss@example.com")).await, PermissionDecision::Ask);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn a_whole_tool_rule_still_covers_every_call(pool: sqlx::PgPool) {
        let user_id = pool_user(&pool).await;
        let mut conn = pool.acquire().await.unwrap();
        remember_decision(&mut conn, user_id, "gmail.send", None, "allow").await.unwrap();
        assert_eq!(check_tool_permission(&mut conn, user_id, "gmail.send", &serde_json::json!({"to": "anyone@x.co"})).await, PermissionDecision::Allow);
    }
}
