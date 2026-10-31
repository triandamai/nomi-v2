//! The chat and inbox endpoints sync reads and writes, with their wire shapes
//! (`backend/crates/nomi-server/src/routes/{sessions,notifications}.rs`), and how they map onto
//! the stored models.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::NomiError;
use crate::network::auth::Auth;
use crate::network::http;
use crate::storage::models::{Message, NotificationItem, Session, STATUS_SENT};

const SESSIONS: &str = "sessions";
const MESSAGES: &str = "messages";
const APPROVAL: &str = "approval";
const NOTIFICATIONS: &str = "notifications";
const READ: &str = "read";
const READ_ALL: &str = "read-all";
const PARAM_LIMIT: &str = "limit";
const PARAM_BEFORE: &str = "before";
const PARAM_PAGE: &str = "page";
const DECISION_APPROVE: &str = "approve";
const DECISION_DENY: &str = "deny";
const NANOS_PER_MILLI: i128 = 1_000_000;

#[derive(Deserialize)]
pub(crate) struct LastMessageDto {
    pub content: String,
}

/// `SessionListItem`.
#[derive(Deserialize)]
pub(crate) struct SessionDto {
    pub id: String,
    pub title: Option<String>,
    pub last_message: Option<LastMessageDto>,
    pub updated_at: String,
}

#[derive(Deserialize)]
struct ListSessionsResponse {
    sessions: Vec<SessionDto>,
}

#[derive(Deserialize)]
struct CreateSessionResponse {
    session_id: String,
}

/// `MessageItem`. `sender` is `user` or `assistant`.
#[derive(Deserialize)]
pub(crate) struct MessageDto {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub content_blocks: Option<Value>,
    pub created_at: String,
    pub agent_display_name: Option<String>,
}

/// `ListMessagesResponse`: oldest first; `has_more` means older messages exist.
#[derive(Deserialize)]
pub(crate) struct MessagePageDto {
    pub messages: Vec<MessageDto>,
    #[serde(default)]
    pub has_more: bool,
}

/// `IngestMessageResponse`: the stored message, and the supervisor's reply to a stop command.
#[derive(Deserialize)]
pub(crate) struct SentMessageDto {
    pub user_message: MessageDto,
    pub supervisor_reply: Option<MessageDto>,
}

#[derive(Serialize)]
struct SendMessageRequest<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct ApprovalRequest {
    decision: &'static str,
    remember: bool,
}

/// `NotificationItem`: the server says only whether it was read, not when.
#[derive(Deserialize)]
pub(crate) struct NotificationDto {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    pub read: bool,
    pub created_at: String,
}

/// `InboxPage`; pages count from 1.
#[derive(Deserialize)]
pub(crate) struct NotificationPageDto {
    pub items: Vec<NotificationDto>,
    pub page: u32,
    pub pages: u32,
}

/// The backend's REST endpoints for chats, messages and the inbox, called with the session.
#[derive(Clone)]
pub(crate) struct Api {
    auth: Arc<Auth>,
}

impl Api {
    pub fn new(auth: Arc<Auth>) -> Self {
        Self { auth }
    }

    pub fn auth(&self) -> &Auth {
        &self.auth
    }

    /// The person's chats in the active org, latest first. Not paginated: it is the whole list.
    pub async fn sessions(&self) -> Result<Vec<SessionDto>, NomiError> {
        let url = self.auth.api_url(&[SESSIONS]);
        let response = self.auth.send(|client| client.get(url.clone())).await?;
        Ok(http::json::<ListSessionsResponse>(response).await?.sessions)
    }

    /// Creates an empty chat; the server lists it once its first message is sent.
    pub async fn create_session(&self) -> Result<String, NomiError> {
        let url = self.auth.api_url(&[SESSIONS]);
        let response = self.auth.send(|client| client.post(url.clone())).await?;
        Ok(http::json::<CreateSessionResponse>(response)
            .await?
            .session_id)
    }

    /// Up to `limit` messages before `before` (the newest without it), oldest first.
    pub async fn messages(
        &self,
        session_id: &str,
        before: Option<&str>,
        limit: u32,
    ) -> Result<MessagePageDto, NomiError> {
        let mut url = self.auth.api_url(&[SESSIONS, session_id, MESSAGES]);
        {
            let mut query = url.query_pairs_mut();
            query.append_pair(PARAM_LIMIT, &limit.to_string());
            if let Some(before) = before {
                query.append_pair(PARAM_BEFORE, before);
            }
        }
        let response = self.auth.send(|client| client.get(url.clone())).await?;
        http::json(response).await
    }

    pub async fn message(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<MessageDto, NomiError> {
        let url = self
            .auth
            .api_url(&[SESSIONS, session_id, MESSAGES, message_id]);
        let response = self.auth.send(|client| client.get(url.clone())).await?;
        http::json(response).await
    }

    /// Posts the person's message; the reply arrives over realtime.
    pub async fn send_message(
        &self,
        session_id: &str,
        text: &str,
    ) -> Result<SentMessageDto, NomiError> {
        let url = self.auth.api_url(&[SESSIONS, session_id, MESSAGES]);
        let body = SendMessageRequest { text };
        let response = self
            .auth
            .send(|client| client.post(url.clone()).json(&body))
            .await?;
        http::json(response).await
    }

    /// Decides a pending approval card; 409 means it is no longer pending.
    pub async fn resolve_approval(
        &self,
        session_id: &str,
        message_id: &str,
        approve: bool,
        remember: bool,
    ) -> Result<(), NomiError> {
        let url = self
            .auth
            .api_url(&[SESSIONS, session_id, MESSAGES, message_id, APPROVAL]);
        let body = ApprovalRequest {
            decision: if approve {
                DECISION_APPROVE
            } else {
                DECISION_DENY
            },
            remember,
        };
        let response = self
            .auth
            .send(|client| client.put(url.clone()).json(&body))
            .await?;
        http::success(response).await.map(drop)
    }

    pub async fn notifications(&self, page: u32) -> Result<NotificationPageDto, NomiError> {
        let mut url = self.auth.api_url(&[NOTIFICATIONS]);
        url.query_pairs_mut()
            .append_pair(PARAM_PAGE, &page.to_string());
        let response = self.auth.send(|client| client.get(url.clone())).await?;
        http::json(response).await
    }

    pub async fn mark_notification_read(&self, notification_id: &str) -> Result<(), NomiError> {
        let url = self.auth.api_url(&[NOTIFICATIONS, notification_id, READ]);
        let response = self.auth.send(|client| client.post(url.clone())).await?;
        http::success(response).await.map(drop)
    }

    pub async fn mark_all_notifications_read(&self) -> Result<(), NomiError> {
        let url = self.auth.api_url(&[NOTIFICATIONS, READ_ALL]);
        let response = self.auth.send(|client| client.post(url.clone())).await?;
        http::success(response).await.map(drop)
    }
}

/// An RFC 3339 timestamp (chrono's `DateTime<Utc>` on the wire) in epoch milliseconds.
pub(crate) fn parse_time(value: &str) -> Result<i64, NomiError> {
    let parsed =
        OffsetDateTime::parse(value, &Rfc3339).map_err(|error| NomiError::InvalidResponse {
            reason: format!("bad timestamp `{value}`: {error}"),
        })?;
    Ok((parsed.unix_timestamp_nanos() / NANOS_PER_MILLI) as i64)
}

impl SessionDto {
    /// The unread flag is the device's own; storage keeps it (see `merge_from_server`).
    pub fn into_session(self) -> Result<Session, NomiError> {
        Ok(Session {
            updated_at: parse_time(&self.updated_at)?,
            id: self.id.to_lowercase(),
            title: self.title.unwrap_or_default(),
            last_message_preview: self.last_message.map(|m| m.content).unwrap_or_default(),
            unread: false,
        })
    }
}

impl MessageDto {
    pub fn into_message(self, session_id: &str) -> Result<Message, NomiError> {
        Ok(Message {
            created_at: parse_time(&self.created_at)?,
            id: self.id.to_lowercase(),
            session_id: session_id.to_owned(),
            role: self.sender,
            // History carries only the display name; the agent type is known for live activity.
            agent_type: None,
            agent_display_name: self.agent_display_name,
            content: self.content,
            content_json: self
                .content_blocks
                .filter(|blocks| !blocks.is_null())
                .map(|blocks| blocks.to_string()),
            status: STATUS_SENT.to_owned(),
        })
    }
}

impl NotificationDto {
    /// A notification the server says was read is stamped `now`; storage keeps an earlier stamp.
    pub fn into_item(self, now: i64) -> Result<NotificationItem, NomiError> {
        Ok(NotificationItem {
            created_at: parse_time(&self.created_at)?,
            id: self.id.to_lowercase(),
            kind: self.kind,
            title: self.title,
            body: self.body,
            link: self.link,
            read_at: self.read.then_some(now),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_backends_timestamps() {
        assert_eq!(parse_time("1970-01-01T00:00:01Z").unwrap(), 1_000);
        assert_eq!(
            parse_time("2026-10-31T12:00:00.123456789Z").unwrap(),
            1_793_448_000_123
        );
        assert_eq!(
            parse_time("2026-10-31T14:00:00.5+02:00").unwrap(),
            1_793_448_000_500
        );
        assert!(matches!(
            parse_time("yesterday"),
            Err(NomiError::InvalidResponse { .. })
        ));
    }

    #[test]
    fn maps_a_message_and_drops_null_blocks() {
        let dto: MessageDto = serde_json::from_value(serde_json::json!({
            "id": "M1", "sender": "assistant", "content": "Hi", "content_blocks": null,
            "created_at": "1970-01-01T00:00:02Z", "agent_display_name": "Coder",
            "my_feedback": null, "memory_count": 0, "steps": [], "sources": [], "versions": 0
        }))
        .unwrap();
        let message = dto.into_message("s1").unwrap();
        assert_eq!(message.id, "m1");
        assert_eq!(message.content_json, None);
        assert_eq!(message.created_at, 2_000);
        assert_eq!(message.status, STATUS_SENT);
    }
}
