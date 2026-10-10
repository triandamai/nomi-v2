//! Requests to build something (an app, a website, a game...) belong to a project: Rena plans
//! it, Koda builds it. On the web, a build request in a regular chat starts a new project chat
//! and moves the request there, leaving a link to it behind; in a project chat it always goes to
//! Rena, whichever agent was talking before.

use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{ContentBlock as RichBlock, TurnError};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::queue;

/// Rena, who plans projects (nomi-agent-planning), and Koda, who builds them (nomi-agent-coding).
pub const PLANNING_AGENT_TYPE: &str = "planning";
pub const CODING_AGENT_TYPE: &str = "coding";

/// Words that ask for something to be made (English and Indonesian).
const MAKE: &[&str] = &[
    "build", "create", "make", "develop", "code", "program", "design", "prototype", "start", "set up", "setup", "generate",
    "bikin", "bikinin", "buat", "buatkan", "buatin", "membuat", "membangun", "bangun", "kembangkan", "rancang",
];

/// Things that are built as a project.
const THINGS: &[&str] = &[
    "app", "apps", "application", "web app", "webapp", "website", "web site", "site", "landing page", "web page", "webpage",
    "homepage", "portfolio", "dashboard", "game", "chrome extension", "extension", "browser extension", "chatbot", "bot",
    "api", "backend", "frontend", "saas", "online store", "online shop", "e-commerce", "ecommerce", "blog", "calculator",
    "todo list", "to-do list", "tracker", "aplikasi", "situs", "halaman web", "laman", "toko online", "permainan",
];

/// Mentions that make it a Google Workspace request instead ("create a Google Doc").
const WORKSPACE: &[&str] = &["google doc", "google sheet", "google slide", "gmail", "google calendar", "google drive", "spreadsheet", "google form"];

/// Whether `phrase` appears in `text` as whole words.
fn has_phrase(text: &str, phrase: &str) -> bool {
    let mut from = 0;
    while let Some(at) = text[from..].find(phrase) {
        let start = from + at;
        let end = start + phrase.len();
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        if !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric) {
            return true;
        }
        from = end;
    }
    false
}

/// Whether the message asks for an app, website or the like to be built.
pub fn wants_to_build(text: &str) -> bool {
    let text = text.to_lowercase();
    if WORKSPACE.iter().any(|w| text.contains(w)) {
        return false;
    }
    MAKE.iter().any(|w| has_phrase(&text, w)) && THINGS.iter().any(|w| has_phrase(&text, w))
}

/// The project this chat belongs to, if it's a project chat.
pub async fn session_project(conn: &mut PoolConnection<Postgres>, session_id: Uuid) -> Result<Option<Uuid>, TurnError> {
    Ok(sqlx::query_scalar("SELECT id FROM projects WHERE session_id = $1 ORDER BY created_at DESC LIMIT 1")
        .bind(session_id)
        .fetch_optional(&mut **conn)
        .await?)
}

/// Whether the chat is on the web (the only place with project pages).
pub async fn is_web_session(conn: &mut PoolConnection<Postgres>, session_id: Uuid) -> Result<bool, TurnError> {
    let channel: Option<String> = sqlx::query_scalar("SELECT channel FROM sessions WHERE id = $1").bind(session_id).fetch_optional(&mut **conn).await?;
    Ok(channel.as_deref() == Some("web"))
}

/// A short project name from the request, until Rena names it properly.
pub fn draft_name(text: &str) -> String {
    let line = text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("New project");
    let mut name: String = line.chars().take(48).collect();
    if line.chars().count() > 48 {
        name = format!("{}…", name.trim_end());
    }
    name
}

/// Starts a project chat for `text`, sends the request there (Rena picks it up as that chat's
/// first message) and leaves a card linking to it in this chat. Returns that card's text and id.
pub async fn move_to_new_project(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
    agent_display_name: &str,
) -> Result<(String, Option<Uuid>), TurnError> {
    let name = draft_name(text);
    let mut tx = sqlx::Connection::begin(&mut **conn).await?;
    let (project_session_id, org_id): (Uuid, Uuid) = sqlx::query_as(
        "INSERT INTO sessions (org_id, user_id, channel, chat_type, chat_id, title) \
         SELECT org_id, user_id, 'web', 'dm', gen_random_uuid()::text, $2 FROM sessions WHERE id = $1 \
         RETURNING id, org_id",
    )
    .bind(session_id)
    .bind(&name)
    .fetch_one(&mut *tx)
    .await?;
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, $3) RETURNING id")
        .bind(user_id)
        .bind(project_session_id)
        .bind(&name)
        .fetch_one(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, $3)")
        .bind(project_session_id)
        .bind(sender_channel_identity_id)
        .bind(text)
        .execute(&mut *tx)
        .await?;
    queue::enqueue(&mut tx, project_session_id, sender_channel_identity_id, text, Some(org_id)).await?;

    let block = RichBlock::ProjectLink { project_id, session_id: project_session_id, name: name.clone() };
    let reply = format!("I started a project for this: {name}. Open it to watch it get planned and built.");
    let message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, agent_display_name) \
         VALUES ($1, NULL, $2, $3, $4) RETURNING id",
    )
    .bind(session_id)
    .bind(&reply)
    .bind(serde_json::json!([block]))
    .bind(agent_display_name)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    if let Some((publisher, _)) = mqtt {
        let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
    }
    Ok((reply, Some(message_id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_requests_are_spotted_in_english_and_indonesian() {
        for text in [
            "create a web app for tracking my habits",
            "Can you build me a website for my bakery?",
            "make a simple todo list app",
            "bikinin aplikasi kasir dong",
            "buatkan website portfolio untuk saya",
            "I want to build a game like snake",
            "set up a landing page for my startup",
        ] {
            assert!(wants_to_build(text), "{text}");
        }
    }

    #[test]
    fn other_requests_are_left_to_the_router() {
        for text in [
            "create a Google Doc with my meeting notes",
            "make a spreadsheet of my expenses",
            "remind me to call mom tomorrow",
            "what's a good app for budgeting?",
            "I spent 50k on lunch",
            "make a plan for my trip to Bali",
            "happy birthday!",
            "the website is down",
        ] {
            assert!(!wants_to_build(text), "{text}");
        }
    }

    #[test]
    fn whole_words_only() {
        assert!(!has_phrase("happy", "app"));
        assert!(!has_phrase("websites are fun", "website"));
        assert!(has_phrase("a web app.", "web app"));
    }

    #[test]
    fn draft_names_are_short_first_lines() {
        assert_eq!(draft_name("  \nBuild a habit tracker\nwith streaks"), "Build a habit tracker");
        let long = "Create a web app that helps small bakeries manage orders, stock and deliveries";
        assert!(draft_name(long).ends_with('…') && draft_name(long).chars().count() <= 49);
    }
}
