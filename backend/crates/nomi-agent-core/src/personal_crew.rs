//! A person's own crew (Settings › Crew): members they switched off, what each member may do for
//! them (allow / ask first / never, by tool), each member's soul.md with its versions, and the
//! templates a member of their own can start from. The router, delegation and the tool loop read
//! these; `nomi_server::routes::agents` writes them.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

use crate::prompts;
use crate::tool_catalog::ToolCatalog;
use crate::tools::ToolSwitches;

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

/// `off` plus Sora unless the design workflow is rolled out to the person (`design_on`): until
/// then she is never routed to, delegated to or asked, like a member they switched off.
pub fn with_design_gate(mut off: HashSet<String>, design_on: bool) -> HashSet<String> {
    if !design_on {
        off.insert(crate::crew::DESIGN_AGENT_TYPE.to_string());
    }
    off
}

/// The members that are off for `user_id`: the ones they switched off, and Sora while the design
/// workflow isn't rolled out to them.
pub async fn disabled_agent_types(conn: &mut PgConnection, user_id: Uuid) -> HashSet<String> {
    let off = sqlx::query_scalar::<_, String>(
        "SELECT agent_type FROM user_agent_settings WHERE user_id = $1 AND NOT enabled",
    )
    .bind(user_id)
    .fetch_all(conn)
    .await
    .unwrap_or_default()
    .into_iter()
    .collect();
    with_design_gate(off, nomi_settings::config::design_enabled_for(user_id).await)
}

/// Whether `agent_type` is on for `user_id` (no row: on). Sora is on only where the design
/// workflow is rolled out to them.
pub async fn is_enabled(conn: &mut PgConnection, user_id: Uuid, agent_type: &str) -> bool {
    if agent_type == crate::crew::DESIGN_AGENT_TYPE && !nomi_settings::config::design_enabled_for(user_id).await {
        return false;
    }
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

/// The longest soul.md a member keeps.
pub const SOUL_MAX_CHARS: usize = 8_000;
/// The most members of their own a person keeps, whatever their plan allows.
pub const MAX_OWN_MEMBERS: i64 = 20;
/// The longest name and role of a member of one's own.
pub const NAME_MAX_CHARS: usize = 40;
pub const ROLE_MAX_CHARS: usize = 200;
/// The look of a member made without picking one.
pub const DEFAULT_SHAPE: &str = "cookie9";
pub const DEFAULT_TONE: &str = "glow";
pub const DEFAULT_MOTION: &str = "spin";

/// A member of the person's own, about to be added (Settings › Crew, or Nomi's `create_agent`).
#[derive(Debug, Clone)]
pub struct NewMember {
    pub name: String,
    pub role: String,
    /// Empty: no soul.md yet.
    pub soul: String,
    pub tools: Vec<String>,
    pub template_key: Option<&'static str>,
    pub shape: String,
    pub tone: String,
    pub motion: String,
}

/// The member `create_member` added.
#[derive(Debug, Clone, PartialEq)]
pub struct CreatedMember {
    pub agent_type: String,
    pub name: String,
    pub role: String,
    pub tools: Vec<String>,
    /// 0 when it has no soul.md.
    pub soul_version: i32,
}

/// Why a member wasn't added. Each carries the text to show the person.
#[derive(Debug)]
pub enum CreateMemberError {
    /// Something in the request is wrong (too long, an unknown tool).
    Invalid(String),
    /// The name is taken in their crew.
    Taken(String),
    /// They already keep `MAX_OWN_MEMBERS`.
    TooMany(String),
    /// Their plan's `max_custom_crew` is reached.
    PlanLimit(String),
    Db(sqlx::Error),
}

impl CreateMemberError {
    pub fn message(&self) -> String {
        match self {
            Self::Invalid(m) | Self::Taken(m) | Self::TooMany(m) | Self::PlanLimit(m) => m.clone(),
            Self::Db(_) => "failed to add the member".to_string(),
        }
    }
}

impl From<sqlx::Error> for CreateMemberError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}

/// The first of `tools` the catalog doesn't have, or an admin switched off, as the error naming
/// it. Only a save is held to this: a member already saved with a tool that is later switched off
/// keeps the row, and the engine skips the tool at run time.
pub fn check_tool_names(catalog: &ToolCatalog, switches: &ToolSwitches, tools: &[String]) -> Result<(), String> {
    let known = catalog.known_tool_names();
    if let Some(tool) = tools.iter().find(|t| !known.contains(&t.as_str())) {
        return Err(format!("unknown tool {tool}"));
    }
    match tools.iter().find(|t| !switches.allows(t)) {
        Some(tool) => Err(format!("the tool {tool} is switched off")),
        None => Ok(()),
    }
}

/// `tools` trimmed, without blanks, each name once, in the order given.
pub fn dedupe_tool_names(tools: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    tools.into_iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty() && seen.insert(t.clone())).collect()
}

/// The lengths a member's name, role and soul.md may have.
pub fn check_member_text(name: &str, role: &str, soul: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("name is required".to_string());
    }
    if role.trim().is_empty() {
        return Err("role is required".to_string());
    }
    if name.chars().count() > NAME_MAX_CHARS {
        return Err(format!("name is at most {NAME_MAX_CHARS} characters"));
    }
    if role.chars().count() > ROLE_MAX_CHARS {
        return Err(format!("role is at most {ROLE_MAX_CHARS} characters"));
    }
    if soul.chars().count() > SOUL_MAX_CHARS {
        return Err(format!("soul.md is at most {SOUL_MAX_CHARS} characters"));
    }
    Ok(())
}

fn taken(name: &str) -> CreateMemberError {
    CreateMemberError::Taken(format!("{name} is already in the crew"))
}

/// Adds `member` to `user_id`'s own crew: only they see it, talk to it and can route to it; its
/// soul.md (when given) starts at version 1. `crew_names` are the built-in members' names: a
/// mention goes by name, so one of theirs would never reach it. Enforces the hard cap and the
/// plan's `max_custom_crew`. The one way a member of one's own is made (`POST /api/agents` and
/// the `create_agent` tool).
pub async fn create_member(
    conn: &mut PgConnection,
    user_id: Uuid,
    member: NewMember,
    catalog: &ToolCatalog,
    crew_names: &[String],
) -> Result<CreatedMember, CreateMemberError> {
    let NewMember { name, role, soul, tools, template_key, shape, tone, motion } = member;
    check_member_text(&name, &role, &soul).map_err(CreateMemberError::Invalid)?;
    let switches = ToolSwitches::load(conn).await;
    check_tool_names(catalog, &switches, &tools).map_err(CreateMemberError::Invalid)?;
    if crew_names.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
        return Err(taken(&name));
    }

    let mut tx = conn.begin().await?;
    let own: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dynamic_agents WHERE owner_user_id = $1")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
    if own >= MAX_OWN_MEMBERS {
        return Err(CreateMemberError::TooMany(format!("a crew keeps at most {MAX_OWN_MEMBERS} members of your own")));
    }
    // The plan's own limit on members of one's own, under the hard cap above.
    if let Err(hit) = nomi_usage::entitlements::check_new_crew_member(&mut tx, user_id).await? {
        return Err(CreateMemberError::PlanLimit(hit.message()));
    }
    let name_taken: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM dynamic_agents WHERE lower(name) = lower($1) AND is_active AND (owner_user_id IS NULL OR owner_user_id = $2))",
    )
    .bind(&name)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;
    if name_taken {
        return Err(taken(&name));
    }
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dynamic_agents (id, name, system_prompt, intent_label, intent_description, granted_tools, created_by, owner_user_id, template_key, shape, tone, motion) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8, $9, $10, $11)",
    )
    .bind(id)
    .bind(&name)
    .bind(personal_member_prompt(&name, &role))
    .bind(intent_label_for(&name, id))
    .bind(&role)
    .bind(&tools)
    .bind(user_id)
    .bind(template_key)
    .bind(&shape)
    .bind(&tone)
    .bind(&motion)
    .execute(&mut *tx)
    .await?;
    let agent_type = id.to_string();
    let soul_version = if soul.is_empty() { 0 } else { save_soul(&mut tx, user_id, &agent_type, &soul).await? };
    tx.commit().await?;
    Ok(CreatedMember { agent_type, name, role, tools, soul_version })
}

/// Replaces the tools of `user_id`'s own member `id`. False when they have no such member.
pub async fn set_member_tools(conn: &mut PgConnection, user_id: Uuid, id: Uuid, tools: &[String]) -> Result<bool, sqlx::Error> {
    let updated = sqlx::query("UPDATE dynamic_agents SET granted_tools = $3, updated_at = now() WHERE id = $1 AND owner_user_id = $2")
        .bind(id)
        .bind(user_id)
        .bind(tools)
        .execute(conn)
        .await?
        .rows_affected();
    Ok(updated > 0)
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
    fn member_text_is_checked_for_presence_and_length() {
        assert!(check_member_text("Chef", "cooks", "").is_ok());
        assert_eq!(check_member_text(" ", "cooks", "").unwrap_err(), "name is required");
        assert_eq!(check_member_text("Chef", "", "").unwrap_err(), "role is required");
        let long_name = "x".repeat(NAME_MAX_CHARS + 1);
        assert!(check_member_text(&long_name, "cooks", "").unwrap_err().starts_with("name is at most"));
        let long_role = "x".repeat(ROLE_MAX_CHARS + 1);
        assert!(check_member_text("Chef", &long_role, "").unwrap_err().starts_with("role is at most"));
        let long_soul = "x".repeat(SOUL_MAX_CHARS + 1);
        assert!(check_member_text("Chef", "cooks", &long_soul).unwrap_err().starts_with("soul.md is at most"));
    }

    #[test]
    fn tool_names_are_checked_against_the_catalog() {
        let catalog = ToolCatalog::empty();
        let switches = ToolSwitches::default();
        assert!(check_tool_names(&catalog, &switches, &[]).is_ok());
        assert_eq!(check_tool_names(&catalog, &switches, &["made_up".to_string()]).unwrap_err(), "unknown tool made_up");
    }

    #[test]
    fn sora_counts_as_switched_off_while_design_is_not_rolled_out_to_the_person() {
        let off: HashSet<String> = ["money".to_string()].into();
        let gated = with_design_gate(off.clone(), false);
        assert!(gated.contains(crate::crew::DESIGN_AGENT_TYPE) && gated.contains("money"));
        assert_eq!(with_design_gate(off.clone(), true), off, "rolled out: only what the person switched off");
    }

    #[test]
    fn switched_off_tools_are_refused_at_save_time() {
        let catalog = ToolCatalog::empty();
        let switches = ToolSwitches::with_off(["made_up"]);
        // Unknown wins over switched off; a required tool is never refused.
        assert_eq!(check_tool_names(&catalog, &switches, &["made_up".to_string()]).unwrap_err(), "unknown tool made_up");
        assert!(!switches.allows("made_up"));
    }

    #[test]
    fn tool_names_are_deduped_in_order() {
        let tools = ["b", " a ", "b", "", "a", "c"].map(String::from);
        assert_eq!(dedupe_tool_names(tools), vec!["b", "a", "c"]);
    }

    #[test]
    fn prompts_fill_their_blanks() {
        assert!(soul_prompt("  be kind ").ends_with("be kind"));
        assert!(personal_member_prompt("Chef", "cooks").contains("You are Chef"));
        assert!(switched_off_result("Dana").starts_with("Dana is switched off"));
    }
}
