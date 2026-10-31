//! The SDK's stored models, as Kotlin sees them. Timestamps are epoch milliseconds.

/// A chat in the list. `title` is empty until the server names the chat.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub last_message_preview: String,
    pub updated_at: i64,
    pub unread: bool,
}

/// One chat message. `role` is `user` or `assistant`; `content` is the markdown text and
/// `content_json` the server's content-blocks JSON array, when the message has one.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub agent_type: Option<String>,
    pub agent_display_name: Option<String>,
    pub content: String,
    pub content_json: Option<String>,
    pub created_at: i64,
    pub status: String,
}

/// The assistant reply streaming in for a chat's current turn.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Draft {
    pub session_id: String,
    pub turn_job_id: String,
    pub text: String,
    pub phase: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct NotificationItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    pub created_at: i64,
    pub read_at: Option<i64>,
}

/// A crew agent alive in a chat. `phase` is the backend's: `thinking`, `calling_tool`, `writing`,
/// or `waiting` (alive but idle).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct AgentActivity {
    pub agent_session_id: String,
    pub session_id: String,
    pub agent_type: String,
    pub phase: String,
    pub detail: Option<String>,
    pub updated_at: i64,
}

/// Which chats to list, newest first.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SessionFilter {
    /// Case-insensitive text matched against the title and the last message preview.
    pub search: Option<String>,
    pub unread_only: bool,
    pub limit: u32,
}

/// Where a page of older messages starts: the oldest message already shown.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct MessageCursor {
    pub created_at: i64,
    pub id: String,
}

/// Messages oldest first, and the cursor for the page before them when there is one.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct MessagePage {
    pub messages: Vec<Message>,
    pub older: Option<MessageCursor>,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DatabaseStatus {
    pub account_id: String,
    pub path: String,
    pub schema_version: u32,
    /// The database file plus its write-ahead log.
    pub size_bytes: u64,
}
