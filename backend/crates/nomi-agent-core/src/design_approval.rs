//! The person's approval of one version of a chat's design previews, given in the Design
//! Studio. Approvals are only ever added (`design_approvals`): the newest is the one that
//! stands, and the earlier ones stay as the record of what was approved when. Sora adds one only
//! for an approval the person gave in the chat, quoting their latest message (approve_version).

use serde::Serialize;
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

/// The most characters in the note an approval carries.
pub const MAX_NOTE_CHARS: usize = 500;
const DAY: &str = "%Y-%m-%d";

/// One approval: the version the person said is the design to build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct Approval {
    #[sqlx(rename = "artifact_version")]
    pub version: i32,
    pub note: Option<String>,
    pub user_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// The note sent with an approval, when there is one.
pub fn note_of(input: &Value) -> Result<Option<String>, String> {
    let note = input.get("note").and_then(Value::as_str).map(str::trim).filter(|note| !note.is_empty());
    match note {
        Some(note) if note.chars().count() > MAX_NOTE_CHARS => Err(format!("a note is at most {MAX_NOTE_CHARS} characters")),
        note => Ok(note.map(str::to_string)),
    }
}

impl Approval {
    /// The approval as Sora reads it, with whether `newest` (the chat's newest preview version)
    /// is the approved one.
    pub fn to_markdown(&self, newest: i32) -> String {
        let note = self.note.as_deref().map(|note| format!(" Their note: {note}")).unwrap_or_default();
        let standing = if newest > self.version {
            format!("Version {newest} is newer and not approved: only version {} may be handed to Koda until they approve another.", self.version)
        } else {
            "It is the newest version.".to_string()
        };
        format!("# Approved design\nThe person approved version {} in the Design Studio on {}.{note}\n{standing}", self.version, self.created_at.format(DAY))
    }
}

const COLUMNS: &str = "artifact_version, note, user_id, created_at";

/// Records that `user_id` approved `version`. Earlier approvals are kept.
pub async fn approve(conn: &mut PgConnection, session_id: Uuid, user_id: Uuid, version: i32, note: Option<&str>) -> Result<Approval, sqlx::Error> {
    sqlx::query_as(&format!("INSERT INTO design_approvals (session_id, user_id, artifact_version, note) VALUES ($1, $2, $3, $4) RETURNING {COLUMNS}"))
        .bind(session_id)
        .bind(user_id)
        .bind(version)
        .bind(note)
        .fetch_one(conn)
        .await
}

/// The approval that stands for the chat: the newest one.
pub async fn current(conn: &mut PgConnection, session_id: Uuid) -> Result<Option<Approval>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM design_approvals WHERE session_id = $1 ORDER BY created_at DESC, id DESC LIMIT 1"))
        .bind(session_id)
        .fetch_optional(conn)
        .await
}

/// Every approval the chat has had, oldest first.
pub async fn history(conn: &mut PgConnection, session_id: Uuid) -> Result<Vec<Approval>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM design_approvals WHERE session_id = $1 ORDER BY created_at, id")).bind(session_id).fetch_all(conn).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn approval(version: i32, note: Option<&str>) -> Approval {
        Approval { version, note: note.map(str::to_string), user_id: Uuid::nil(), created_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH }
    }

    #[test]
    fn an_approval_can_carry_a_short_note() {
        assert_eq!(note_of(&json!({"note": "  Go with this one  "})), Ok(Some("Go with this one".to_string())));
        assert_eq!(note_of(&json!({"note": " "})), Ok(None));
        assert_eq!(note_of(&json!({})), Ok(None));
        assert_eq!(note_of(&json!({"note": "x".repeat(MAX_NOTE_CHARS + 1)})).unwrap_err(), format!("a note is at most {MAX_NOTE_CHARS} characters"));
    }

    #[test]
    fn sora_is_told_which_version_stands_and_whether_a_newer_one_is_waiting() {
        let said = approval(2, Some("Go with this one")).to_markdown(2);
        assert_eq!(said, "# Approved design\nThe person approved version 2 in the Design Studio on 1970-01-01. Their note: Go with this one\nIt is the newest version.");
        let behind = approval(2, None).to_markdown(4);
        assert!(behind.ends_with("Version 4 is newer and not approved: only version 2 may be handed to Koda until they approve another."), "{behind}");
        assert!(!behind.contains("Their note"));
    }

    #[test]
    fn an_approval_goes_to_the_studio_with_its_version_and_when() {
        let sent = serde_json::to_value(approval(3, None)).unwrap();
        assert_eq!((sent["version"].as_i64(), sent["note"].is_null(), sent["created_at"].is_string()), (Some(3), true, true));
    }
}
