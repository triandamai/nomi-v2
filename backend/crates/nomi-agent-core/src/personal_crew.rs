//! A person's own crew (Settings › Crew): members they switched off, what each member may do for
//! them (allow / ask first / never, by tool), each member's soul.md with its versions, and the
//! templates a member of their own can start from. The router, delegation and the tool loop read
//! these; `nomi_server::routes::agents` writes them.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::prompts;

/// What a member may do with one tool, for this person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Runs without an approval card.
    Allow,
    /// Always goes through the approval card.
    Ask,
    /// Refused.
    Never,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Allow => "allow",
            Level::Ask => "ask",
            Level::Never => "never",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "allow" => Some(Level::Allow),
            "ask" => Some(Level::Ask),
            "never" => Some(Level::Never),
            _ => None,
        }
    }
}

/// The members `user_id` switched off.
pub async fn disabled_agent_types(conn: &mut PgConnection, user_id: Uuid) -> HashSet<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT agent_type FROM user_agent_settings WHERE user_id = $1 AND NOT enabled",
    )
    .bind(user_id)
    .fetch_all(conn)
    .await
    .unwrap_or_default()
    .into_iter()
    .collect()
}

/// Whether `agent_type` is on for `user_id` (no row: on).
pub async fn is_enabled(conn: &mut PgConnection, user_id: Uuid, agent_type: &str) -> bool {
    let enabled: Option<bool> = sqlx::query_scalar(
        "SELECT enabled FROM user_agent_settings WHERE user_id = $1 AND agent_type = $2",
    )
    .bind(user_id)
    .bind(agent_type)
    .fetch_optional(conn)
    .await
    .ok()
    .flatten();
    enabled.unwrap_or(true)
}

pub async fn set_enabled(
    conn: &mut PgConnection,
    user_id: Uuid,
    agent_type: &str,
    enabled: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_agent_settings (user_id, agent_type, enabled) VALUES ($1, $2, $3) \
         ON CONFLICT (user_id, agent_type) DO UPDATE SET enabled = EXCLUDED.enabled, updated_at = now()",
    )
    .bind(user_id)
    .bind(agent_type)
    .bind(enabled)
    .execute(conn)
    .await?;
    Ok(())
}

/// What `user_id` decided for `agent_type`, by tool. Tools not in the map keep the member's default.
pub async fn member_levels(
    conn: &mut PgConnection,
    user_id: Uuid,
    agent_type: &str,
) -> HashMap<String, Level> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT key, level FROM user_agent_permissions WHERE user_id = $1 AND agent_type = $2",
    )
    .bind(user_id)
    .bind(agent_type)
    .fetch_all(conn)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .filter_map(|(key, level)| Level::parse(&level).map(|l| (key, l)))
        .collect()
}

/// Records `levels` for `agent_type`, replacing what was decided for those keys before.
pub async fn set_levels(
    conn: &mut PgConnection,
    user_id: Uuid,
    agent_type: &str,
    levels: &[(String, Level)],
) -> Result<(), sqlx::Error> {
    for (key, level) in levels {
        sqlx::query(
            "INSERT INTO user_agent_permissions (user_id, agent_type, key, level) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (user_id, agent_type, key) DO UPDATE SET level = EXCLUDED.level, updated_at = now()",
        )
        .bind(user_id)
        .bind(agent_type)
        .bind(key)
        .bind(level.as_str())
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// The soul.md in use for `agent_type` and its version, when the person wrote one.
pub async fn current_soul(
    conn: &mut PgConnection,
    user_id: Uuid,
    agent_type: &str,
) -> Result<Option<(String, i32)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT v.text, v.version FROM user_agent_souls s \
         JOIN user_agent_soul_versions v ON v.user_id = s.user_id AND v.agent_type = s.agent_type AND v.version = s.current_version \
         WHERE s.user_id = $1 AND s.agent_type = $2",
    )
    .bind(user_id)
    .bind(agent_type)
    .fetch_optional(conn)
    .await
}

/// Saves `text` as the next version of `agent_type`'s soul.md and puts it in use; returns its version.
pub async fn save_soul(
    conn: &mut PgConnection,
    user_id: Uuid,
    agent_type: &str,
    text: &str,
) -> Result<i32, sqlx::Error> {
    let version: i32 = sqlx::query_scalar(
        "INSERT INTO user_agent_soul_versions (user_id, agent_type, version, text) \
         SELECT $1, $2, COALESCE(MAX(version), 0) + 1, $3 FROM user_agent_soul_versions WHERE user_id = $1 AND agent_type = $2 \
         RETURNING version",
    )
    .bind(user_id)
    .bind(agent_type)
    .bind(text)
    .fetch_one(&mut *conn)
    .await?;
    sqlx::query(
        "INSERT INTO user_agent_souls (user_id, agent_type, current_version) VALUES ($1, $2, $3) \
         ON CONFLICT (user_id, agent_type) DO UPDATE SET current_version = EXCLUDED.current_version, updated_at = now()",
    )
    .bind(user_id)
    .bind(agent_type)
    .bind(version)
    .execute(conn)
    .await?;
    Ok(version)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SoulVersion {
    pub version: i32,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub is_current: bool,
}

/// `agent_type`'s soul.md versions, newest first.
pub async fn soul_versions(
    conn: &mut PgConnection,
    user_id: Uuid,
    agent_type: &str,
    limit: i64,
) -> Result<Vec<SoulVersion>, sqlx::Error> {
    sqlx::query_as(
        "SELECT v.version, v.text, v.created_at, (v.version = s.current_version) AS is_current \
         FROM user_agent_soul_versions v \
         LEFT JOIN user_agent_souls s ON s.user_id = v.user_id AND s.agent_type = v.agent_type \
         WHERE v.user_id = $1 AND v.agent_type = $2 ORDER BY v.version DESC LIMIT $3",
    )
    .bind(user_id)
    .bind(agent_type)
    .bind(limit)
    .fetch_all(conn)
    .await
}

/// Puts an earlier version back in use, the way the personality's rollback does. False when
/// there is no such version.
pub async fn restore_soul(
    conn: &mut PgConnection,
    user_id: Uuid,
    agent_type: &str,
    version: i32,
) -> Result<bool, sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE user_agent_souls s SET current_version = $3, updated_at = now() \
         WHERE s.user_id = $1 AND s.agent_type = $2 \
           AND EXISTS (SELECT 1 FROM user_agent_soul_versions v WHERE v.user_id = $1 AND v.agent_type = $2 AND v.version = $3)",
    )
    .bind(user_id)
    .bind(agent_type)
    .bind(version)
    .execute(conn)
    .await?
    .rows_affected();
    Ok(updated > 0)
}

/// The soul.md, as it goes into the member's instructions.
pub fn soul_prompt(soul: &str) -> String {
    prompts::MEMBER_SOUL_TEMPLATE.replace("{soul}", soul.trim())
}

/// The instructions of a member a person made for themselves.
pub fn personal_member_prompt(name: &str, role: &str) -> String {
    prompts::PERSONAL_MEMBER_SYSTEM_PROMPT_TEMPLATE
        .replace("{name}", name.trim())
        .replace("{role}", role.trim())
}

/// What a delegating or asking member reads when the member it named is switched off.
pub fn switched_off_result(name: &str) -> String {
    prompts::MEMBER_SWITCHED_OFF_RESULT.replace("{name}", name)
}

/// How many characters of a member's name go into its routing label.
const LABEL_NAME_CHARS: usize = 24;
/// How many hex characters of its id keep the label unique.
const LABEL_ID_CHARS: usize = 8;

/// The one-word routing label of a person's own member: its name's letters and digits,
/// lowercased, then a piece of its id so two people's "Chef" never clash. One alphanumeric word,
/// so the router's whole-word match finds it in a chatty classifier reply too.
pub fn intent_label_for(name: &str, id: Uuid) -> String {
    let mut slug: String = name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(LABEL_NAME_CHARS)
        .collect();
    if !slug.starts_with(|c: char| c.is_ascii_alphabetic()) {
        slug = format!("member{slug}");
    }
    let id: String = id
        .simple()
        .to_string()
        .chars()
        .take(LABEL_ID_CHARS)
        .collect();
    format!("{slug}{id}")
}

/// A starting point for a member of one's own.
#[derive(Debug, Clone, Serialize)]
pub struct CrewTemplate {
    pub key: &'static str,
    pub name: &'static str,
    pub role: &'static str,
    pub soul: &'static str,
    /// Catalog tools it starts with.
    pub tools: &'static [&'static str],
    pub shape: &'static str,
    pub tone: &'static str,
    pub motion: &'static str,
}

pub const TEMPLATES: &[CrewTemplate] = &[
    CrewTemplate {
        key: "budget_coach",
        name: "Budget coach",
        role: "Keeps your spending on track against your budgets",
        soul: prompts::TEMPLATE_SOUL_BUDGET_COACH,
        tools: &[
            "list_transactions",
            "summarize_budget",
            "list_budgets",
            "set_budget",
        ],
        shape: "clover4",
        tone: "citrus",
        motion: "wobble",
    },
    CrewTemplate {
        key: "expense_logger",
        name: "Expense logger",
        role: "Writes down what you paid, as you tell it",
        soul: prompts::TEMPLATE_SOUL_EXPENSE_LOGGER,
        tools: &["log_transaction", "list_transactions"],
        shape: "pill",
        tone: "tide",
        motion: "pulse",
    },
    CrewTemplate {
        key: "writing_buddy",
        name: "Writing buddy",
        role: "Helps you write captions, messages and short posts",
        soul: prompts::TEMPLATE_SOUL_WRITING_BUDDY,
        tools: &[],
        shape: "flower5",
        tone: "bloom",
        motion: "bounce",
    },
    CrewTemplate {
        key: "study_partner",
        name: "Study partner",
        role: "Explains things step by step and quizzes you",
        soul: prompts::TEMPLATE_SOUL_STUDY_PARTNER,
        tools: &[],
        shape: "sunny8",
        tone: "sky",
        motion: "orbit",
    },
];

pub fn template(key: &str) -> Option<&'static CrewTemplate> {
    TEMPLATES.iter().find(|t| t.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_round_trip() {
        for level in [Level::Allow, Level::Ask, Level::Never] {
            assert_eq!(Level::parse(level.as_str()), Some(level));
        }
        assert_eq!(Level::parse("maybe"), None);
    }

    #[test]
    fn labels_are_one_lowercase_word_with_a_piece_of_the_id() {
        let id = Uuid::from_u128(0xabcdef12_0000_0000_0000_000000000000);
        assert_eq!(intent_label_for("Budget Coach!", id), "budgetcoachabcdef12");
        assert_eq!(intent_label_for("42", id), "member42abcdef12");
        assert_eq!(intent_label_for("✨", id), "memberabcdef12");
    }

    #[test]
    fn templates_have_unique_keys_and_a_soul() {
        let mut keys: Vec<&str> = TEMPLATES.iter().map(|t| t.key).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), TEMPLATES.len());
        assert!(TEMPLATES
            .iter()
            .all(|t| !t.soul.is_empty() && template(t.key).is_some()));
    }

    #[test]
    fn prompts_fill_their_blanks() {
        assert!(soul_prompt("  be kind ").ends_with("be kind"));
        assert!(personal_member_prompt("Chef", "cooks").contains("You are Chef"));
        assert!(switched_off_result("Dana").starts_with("Dana is switched off"));
    }
}
