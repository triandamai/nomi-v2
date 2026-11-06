//! The reminder tools past adding one: listing, editing, ticking off, removing, snoozing, and
//! their refusals; and a one-off reminder firing.

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_reminders::{fire_due, snooze, user_timezone, RemindersAgent};

struct Person {
    pool: PgPool,
    user: Uuid,
    session: Uuid,
}

async fn person(pool: PgPool, timezone: Option<&str>) -> Person {
    let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', $2) RETURNING id").bind(org).bind(user.to_string()).fetch_one(&pool).await.unwrap();
    if let Some(tz) = timezone {
        sqlx::query("INSERT INTO user_preferences (user_id, timezone) VALUES ($1, $2)").bind(user).bind(tz).execute(&pool).await.unwrap();
    }
    Person { pool, user, session }
}

impl Person {
    async fn run(&self, tool: &str, input: Value) -> Result<String, String> {
        let mut conn = self.pool.acquire().await.unwrap();
        RemindersAgent.execute_tool(&mut conn, self.session, self.session, self.user, tool, input).await.map(|o| o.display_text)
    }

    async fn add(&self, title: &str, input: Value) -> Uuid {
        let mut body = json!({"title": title, "due_at": (Utc::now() + Duration::days(2)).to_rfc3339()});
        body.as_object_mut().unwrap().extend(input.as_object().unwrap().clone());
        let text = self.run("add_reminder", body).await.unwrap();
        text.trim_start_matches("Reminder set: ").split(" | ").next().unwrap().parse().unwrap()
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn adding_refuses_what_cannot_be_a_reminder(pool: PgPool) {
    let p = person(pool, None).await;
    let future = (Utc::now() + Duration::days(1)).to_rfc3339();
    assert_eq!(p.run("add_reminder", json!({"title": "x"})).await.unwrap_err(), "due_at is required");
    assert_eq!(p.run("add_reminder", json!({"title": "  ", "due_at": future})).await.unwrap_err(), "title is required");
    assert_eq!(p.run("add_reminder", json!({"title": "x", "due_at": "2020-01-01T00:00:00Z"})).await.unwrap_err(), "due_at must be in the future");
    assert_eq!(p.run("add_reminder", json!({"title": "x", "due_at": "soon"})).await.unwrap_err(), "due_at must be an ISO 8601 date and time");
    assert!(p.run("add_reminder", json!({"title": "x", "due_at": future, "recurrence": "hourly"})).await.unwrap_err().contains("not hourly"));
    assert_eq!(p.run("show_reminders", json!({})).await.unwrap(), "No upcoming reminders.");
    assert_eq!(p.run("snooze_everything", json!({})).await.unwrap_err(), "unknown tool: snooze_everything");
    // Without a saved timezone, UTC.
    let mut conn = p.pool.acquire().await.unwrap();
    assert_eq!(user_timezone(&mut conn, p.user).await, chrono_tz::UTC);
}

#[sqlx::test(migrations = "../../migrations")]
async fn reminders_are_listed_edited_ticked_off_and_removed(pool: PgPool) {
    let p = person(pool, Some("Asia/Jakarta")).await;
    let rent = p.add("Pay rent", json!({"recurrence": "monthly", "notes": " transfer to Bu Sri "})).await;
    let gym = p.add("Gym", json!({"recurrence": "none"})).await;

    let shown = p.run("show_reminders", json!({})).await.unwrap();
    assert_eq!(shown.lines().count(), 2);
    assert!(shown.contains("Pay rent") && shown.contains("repeats monthly"), "{shown}");

    let edited = p.run("edit_reminder", json!({"id": gym.to_string(), "title": "Gym (legs)", "due_at": "2099-03-02 07:30", "recurrence": "weekly"})).await.unwrap();
    assert_eq!(edited, "Updated: Gym (legs), Mon 2 Mar 07:30.");
    let (recurrence, weekday): (Option<String>, Option<i16>) = sqlx::query_as("SELECT recurrence, recurrence_weekday FROM reminders WHERE id = $1").bind(gym).fetch_one(&p.pool).await.unwrap();
    assert_eq!((recurrence.as_deref(), weekday), (Some("weekly"), Some(1)));

    // Leaving fields out keeps them.
    let kept = p.run("edit_reminder", json!({"id": rent.to_string(), "title": " "})).await.unwrap();
    assert!(kept.starts_with("Updated: Pay rent,"), "{kept}");
    let notes: Option<String> = sqlx::query_scalar("SELECT notes FROM reminders WHERE id = $1").bind(rent).fetch_one(&p.pool).await.unwrap();
    assert_eq!(notes.as_deref(), Some("transfer to Bu Sri"));

    assert_eq!(p.run("complete_reminder", json!({"id": rent.to_string()})).await.unwrap(), "Ticked off.");
    assert_eq!(p.run("complete_reminder", json!({"id": rent.to_string()})).await.unwrap_err(), "no open reminder with that id");
    assert_eq!(p.run("edit_reminder", json!({"id": rent.to_string()})).await.unwrap_err(), "no open reminder with that id");
    assert_eq!(p.run("remove_reminder", json!({"id": gym.to_string()})).await.unwrap(), "Removed.");
    assert_eq!(p.run("remove_reminder", json!({"id": "not-an-id"})).await.unwrap_err(), "id must be a reminder id from show_reminders");
    assert_eq!(p.run("show_reminders", json!({})).await.unwrap(), "No upcoming reminders.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn someone_elses_reminder_cannot_be_touched(pool: PgPool) {
    let ana = person(pool.clone(), None).await;
    let ben = person(pool, None).await;
    let reminder = ana.add("Ana's thing", json!({})).await;
    assert!(ben.run("complete_reminder", json!({"id": reminder.to_string()})).await.is_err());
    assert!(ben.run("edit_reminder", json!({"id": reminder.to_string(), "title": "mine"})).await.is_err());
    let mut conn = ben.pool.acquire().await.unwrap();
    assert!(!snooze(&mut conn, ben.user, reminder, 10).await.unwrap());
    assert!(snooze(&mut conn, ana.user, reminder, 10).await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_one_off_reminder_fires_once_with_its_notes_and_can_be_snoozed_open_again(pool: PgPool) {
    let p = person(pool, None).await;
    let id = p.add("Take pills", json!({"notes": "the blue ones"})).await;
    sqlx::query("UPDATE reminders SET due_at = now() - interval '1 minute' WHERE id = $1").bind(id).execute(&p.pool).await.unwrap();

    let posted = fire_due(&p.pool, 10).await.unwrap();
    assert_eq!(posted.len(), 1);
    let content: String = sqlx::query_scalar("SELECT content FROM messages WHERE id = $1").bind(posted[0].1).fetch_one(&p.pool).await.unwrap();
    assert_eq!(content, "⏰ Take pills: the blue ones");
    let status: String = sqlx::query_scalar("SELECT status FROM reminders WHERE id = $1").bind(id).fetch_one(&p.pool).await.unwrap();
    assert_eq!(status, "fired");
    assert!(fire_due(&p.pool, 10).await.unwrap().is_empty(), "fired once");

    let mut conn = p.pool.acquire().await.unwrap();
    assert!(snooze(&mut conn, p.user, id, 100_000).await.unwrap(), "a fired one-off can be snoozed");
    let (status, due): (String, chrono::DateTime<Utc>) = sqlx::query_as("SELECT status, due_at FROM reminders WHERE id = $1").bind(id).fetch_one(&p.pool).await.unwrap();
    assert_eq!(status, "active");
    assert!(due < Utc::now() + Duration::days(8), "snoozing is capped at a week");
}

#[test]
fn the_agent_describes_itself_and_needs_no_approval_for_its_own_tools() {
    let agent = RemindersAgent;
    assert_eq!(agent.agent_type(), "reminders");
    assert!(!agent.display_name().is_empty());
    assert!(!agent.system_prompt().is_empty());
    assert_eq!(agent.intent_label(), "reminders");
    assert!(agent.intent_description().contains("remind"));
    assert!(agent.uses_memory() && agent.wants_current_time());
    for tool in agent.tools() {
        assert!(!agent.tool_needs_approval(&tool.name), "{}", tool.name);
    }
    assert!(agent.tool_needs_approval("something_else"));
}
