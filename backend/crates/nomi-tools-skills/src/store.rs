//! The skills a person added to a member, in `user_agent_skills` and its references. Every query
//! is scoped to one person and one member, so nobody ever sees another person's skills.

use chrono::{DateTime, Utc};
use nomi_skills::{ParsedSkill, Reference, SkillEntry, MAX_SKILLS_PER_MEMBER};
use serde::Serialize;
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

/// Postgres' code for a unique violation.
const UNIQUE_VIOLATION: &str = "23505";

/// How a skill came in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    Typed,
    Upload,
    Link,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::Typed => "typed",
            Origin::Upload => "upload",
            Origin::Link => "link",
        }
    }

    fn parse(text: &str) -> Self {
        match text {
            "upload" => Origin::Upload,
            "link" => Origin::Link,
            _ => Origin::Typed,
        }
    }
}

/// One of a person's skills, without its references' text.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub license: String,
    pub source: String,
    pub text: String,
    pub enabled: bool,
    pub origin: Origin,
    pub source_url: Option<String>,
    pub reference_names: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct Raw {
    id: Uuid,
    name: String,
    description: String,
    license: String,
    source: String,
    text: String,
    enabled: bool,
    origin: String,
    source_url: Option<String>,
    reference_names: Vec<String>,
    updated_at: DateTime<Utc>,
}

impl From<Raw> for SkillRow {
    fn from(raw: Raw) -> Self {
        SkillRow {
            id: raw.id,
            name: raw.name,
            description: raw.description,
            license: raw.license,
            source: raw.source,
            text: raw.text,
            enabled: raw.enabled,
            origin: Origin::parse(&raw.origin),
            source_url: raw.source_url,
            reference_names: raw.reference_names,
            updated_at: raw.updated_at,
        }
    }
}

/// Why a write didn't land.
#[derive(Debug)]
pub enum StoreError {
    /// The member already has a skill of that name for this person.
    NameTaken,
    /// The member already has `MAX_SKILLS_PER_MEMBER` skills for this person.
    TooMany,
    /// No such skill for this person and member.
    NotFound,
    Db(sqlx::Error),
}

impl From<sqlx::Error> for StoreError {
    fn from(e: sqlx::Error) -> Self {
        if e.as_database_error().and_then(|d| d.code()).is_some_and(|c| c == UNIQUE_VIOLATION) {
            StoreError::NameTaken
        } else {
            StoreError::Db(e)
        }
    }
}

const SELECT: &str = "SELECT s.id, s.name, s.description, s.license, s.source, s.text, s.enabled, s.origin, s.source_url, \
     COALESCE((SELECT array_agg(r.name ORDER BY r.name) FROM user_agent_skill_references r WHERE r.skill_id = s.id), '{}') AS reference_names, \
     s.updated_at FROM user_agent_skills s";

/// The person's skills for the member, by name.
pub async fn list(conn: &mut PgConnection, user_id: Uuid, agent_type: &str) -> Result<Vec<SkillRow>, sqlx::Error> {
    let rows: Vec<Raw> = sqlx::query_as(&format!("{SELECT} WHERE s.user_id = $1 AND s.agent_type = $2 ORDER BY s.name"))
        .bind(user_id)
        .bind(agent_type)
        .fetch_all(conn)
        .await?;
    Ok(rows.into_iter().map(SkillRow::from).collect())
}

async fn references(conn: &mut PgConnection, skill_id: Uuid) -> Result<Vec<Reference>, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT name, text FROM user_agent_skill_references WHERE skill_id = $1 ORDER BY name")
        .bind(skill_id)
        .fetch_all(conn)
        .await?;
    Ok(rows.into_iter().map(|(name, text)| Reference { name, text }).collect())
}

/// One skill whole, with its references.
pub async fn get(conn: &mut PgConnection, user_id: Uuid, agent_type: &str, id: Uuid) -> Result<Option<(SkillRow, Vec<Reference>)>, sqlx::Error> {
    let raw: Option<Raw> = sqlx::query_as(&format!("{SELECT} WHERE s.user_id = $1 AND s.agent_type = $2 AND s.id = $3"))
        .bind(user_id)
        .bind(agent_type)
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;
    match raw {
        Some(raw) => {
            let refs = references(conn, raw.id).await?;
            Ok(Some((raw.into(), refs)))
        }
        None => Ok(None),
    }
}

/// The person's switched-on skill of that name for the member, with its references.
pub async fn find_enabled(conn: &mut PgConnection, user_id: Uuid, agent_type: &str, name: &str) -> Result<Option<(SkillRow, Vec<Reference>)>, sqlx::Error> {
    let raw: Option<Raw> = sqlx::query_as(&format!("{SELECT} WHERE s.user_id = $1 AND s.agent_type = $2 AND s.name = $3 AND s.enabled"))
        .bind(user_id)
        .bind(agent_type)
        .bind(name)
        .fetch_optional(&mut *conn)
        .await?;
    match raw {
        Some(raw) => {
            let refs = references(conn, raw.id).await?;
            Ok(Some((raw.into(), refs)))
        }
        None => Ok(None),
    }
}

/// The member's index for this person: switched-on skills, by name.
pub async fn index(conn: &mut PgConnection, user_id: Uuid, agent_type: &str) -> Result<Vec<SkillEntry>, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT name, description FROM user_agent_skills WHERE user_id = $1 AND agent_type = $2 AND enabled ORDER BY name",
    )
    .bind(user_id)
    .bind(agent_type)
    .fetch_all(conn)
    .await?;
    Ok(rows.into_iter().map(|(name, description)| SkillEntry { name, description }).collect())
}

/// `source` as stored: what the frontmatter says, else the link it came from.
fn source_of(skill: &ParsedSkill, source_url: Option<&str>) -> String {
    if skill.meta.source.trim().is_empty() {
        source_url.unwrap_or_default().to_string()
    } else {
        skill.meta.source.clone()
    }
}

async fn write_references(conn: &mut PgConnection, skill_id: Uuid, refs: &[Reference]) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_agent_skill_references WHERE skill_id = $1").bind(skill_id).execute(&mut *conn).await?;
    for reference in refs {
        sqlx::query("INSERT INTO user_agent_skill_references (skill_id, name, text) VALUES ($1, $2, $3)")
            .bind(skill_id)
            .bind(&reference.name)
            .bind(&reference.text)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

async fn row(conn: &mut PgConnection, user_id: Uuid, agent_type: &str, id: Uuid) -> Result<SkillRow, StoreError> {
    get(conn, user_id, agent_type, id).await?.map(|(row, _)| row).ok_or(StoreError::NotFound)
}

/// Adds a skill to the member for this person. The count and the insert run under one lock per
/// person and member, so two adds at the cap can't both land.
pub async fn insert(
    conn: &mut PgConnection,
    user_id: Uuid,
    agent_type: &str,
    skill: &ParsedSkill,
    origin: Origin,
    source_url: Option<&str>,
) -> Result<SkillRow, StoreError> {
    let mut tx = conn.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1::text || ':' || $2))")
        .bind(user_id)
        .bind(agent_type)
        .execute(&mut *tx)
        .await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM user_agent_skills WHERE user_id = $1 AND agent_type = $2")
        .bind(user_id)
        .bind(agent_type)
        .fetch_one(&mut *tx)
        .await?;
    if count >= MAX_SKILLS_PER_MEMBER {
        return Err(StoreError::TooMany);
    }
    let id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO user_agent_skills (user_id, agent_type, name, description, license, source, text, origin, source_url) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (user_id, agent_type, name) DO NOTHING RETURNING id",
    )
    .bind(user_id)
    .bind(agent_type)
    .bind(&skill.meta.name)
    .bind(&skill.meta.description)
    .bind(&skill.meta.license)
    .bind(source_of(skill, source_url))
    .bind(&skill.text)
    .bind(origin.as_str())
    .bind(source_url)
    .fetch_optional(&mut *tx)
    .await?;
    let id = id.ok_or(StoreError::NameTaken)?;
    write_references(&mut tx, id, &skill.references).await?;
    tx.commit().await?;
    row(conn, user_id, agent_type, id).await
}

/// Replaces a skill's text and references; a new name is taken when it is free. `source_url`, when
/// given, replaces the link it came from; `origin`, when given, replaces how it came in.
pub async fn replace(
    conn: &mut PgConnection,
    user_id: Uuid,
    agent_type: &str,
    id: Uuid,
    skill: &ParsedSkill,
    source_url: Option<&str>,
    origin: Option<Origin>,
) -> Result<SkillRow, StoreError> {
    let mut tx = conn.begin().await?;
    let current: Option<Option<String>> = sqlx::query_scalar(
        "SELECT source_url FROM user_agent_skills WHERE user_id = $1 AND agent_type = $2 AND id = $3 FOR UPDATE",
    )
    .bind(user_id)
    .bind(agent_type)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(current_url) = current else { return Err(StoreError::NotFound) };
    let url = source_url.map(str::to_string).or(current_url);
    sqlx::query(
        "UPDATE user_agent_skills SET name = $2, description = $3, license = $4, source = $5, text = $6, source_url = $7, \
         origin = COALESCE($8, origin), updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(&skill.meta.name)
    .bind(&skill.meta.description)
    .bind(&skill.meta.license)
    .bind(source_of(skill, url.as_deref()))
    .bind(&skill.text)
    .bind(&url)
    .bind(origin.map(Origin::as_str))
    .execute(&mut *tx)
    .await?;
    write_references(&mut tx, id, &skill.references).await?;
    tx.commit().await?;
    row(conn, user_id, agent_type, id).await
}

/// Switches a skill on or off; false when there is no such skill.
pub async fn set_enabled(conn: &mut PgConnection, user_id: Uuid, agent_type: &str, id: Uuid, enabled: bool) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("UPDATE user_agent_skills SET enabled = $4, updated_at = now() WHERE user_id = $1 AND agent_type = $2 AND id = $3")
        .bind(user_id)
        .bind(agent_type)
        .bind(id)
        .bind(enabled)
        .execute(conn)
        .await?;
    Ok(done.rows_affected() > 0)
}

/// Removes a skill; false when there is no such skill.
pub async fn delete(conn: &mut PgConnection, user_id: Uuid, agent_type: &str, id: Uuid) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("DELETE FROM user_agent_skills WHERE user_id = $1 AND agent_type = $2 AND id = $3")
        .bind(user_id)
        .bind(agent_type)
        .bind(id)
        .execute(conn)
        .await?;
    Ok(done.rows_affected() > 0)
}

/// Removes every skill the person added to the member (when they delete a member of their own).
pub async fn delete_member(conn: &mut PgConnection, user_id: Uuid, agent_type: &str) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("DELETE FROM user_agent_skills WHERE user_id = $1 AND agent_type = $2")
        .bind(user_id)
        .bind(agent_type)
        .execute(conn)
        .await?;
    Ok(done.rows_affected())
}
