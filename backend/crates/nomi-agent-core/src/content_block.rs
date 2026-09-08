use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub status: TodoStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableVariant {
    Data,
    Comparison,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
}

/// Structured widget data attached to a chat message alongside its plain-text `content` fallback
/// (see `messages.content_blocks`). Named `ContentBlock` to match the design spec; callers that
/// also need `nomi_llm::ContentBlock` (a different type — LLM message content, not chat-UI
/// widgets) in the same scope should import this one aliased, e.g.
/// `use nomi_agent_core::ContentBlock as RichBlock;` (see `engine.rs`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentBlock {
    FileWrite {
        project_id: Uuid,
        path: String,
        content: String,
        previous_content: Option<String>,
    },
    FileDelete {
        project_id: Uuid,
        path: String,
    },
    TodoList {
        items: Vec<TodoItem>,
    },
    Table {
        variant: TableVariant,
        columns: Vec<TableColumn>,
        rows: Vec<Value>,
    },
    ApprovalRequest {
        id: Uuid,
        tool_name: String,
        description: String,
        input: Value,
        status: ApprovalStatus,
        decided_at: Option<DateTime<Utc>>,
    },
}

/// What `SubAgent::execute_tool` returns on success — `display_text` is the plain-text mirror
/// stored in `messages.content` (and used for anything that reads `content` directly: search,
/// memory extraction, notifications); `block` is the structured widget, when this call warrants
/// one. Most tools return `block: None`.
#[derive(Debug, Clone)]
pub struct ToolOutcome {
    pub display_text: String,
    pub block: Option<ContentBlock>,
}

impl ToolOutcome {
    /// Convenience for the common case (no block) — most agents' tools just wrap their existing
    /// return string in this.
    pub fn text(display_text: impl Into<String>) -> Self {
        Self { display_text: display_text.into(), block: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_write_serializes_with_a_kind_tag_and_snake_case_fields() {
        let block = ContentBlock::FileWrite {
            project_id: uuid::Uuid::nil(),
            path: "index.html".to_string(),
            content: "<h1>hi</h1>".to_string(),
            previous_content: Some("<h1>old</h1>".to_string()),
        };
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["kind"], "file_write");
        assert_eq!(json["path"], "index.html");
        assert_eq!(json["previous_content"], "<h1>old</h1>");
    }

    #[test]
    fn file_write_with_no_previous_content_serializes_previous_content_as_null() {
        let block = ContentBlock::FileWrite {
            project_id: uuid::Uuid::nil(),
            path: "new.js".to_string(),
            content: "x".to_string(),
            previous_content: None,
        };
        let json = serde_json::to_value(&block).unwrap();
        assert!(json["previous_content"].is_null());
    }

    #[test]
    fn todo_status_serializes_as_snake_case_strings() {
        assert_eq!(serde_json::to_value(TodoStatus::InProgress).unwrap(), "in_progress");
        assert_eq!(serde_json::to_value(TodoStatus::Done).unwrap(), "done");
    }

    #[test]
    fn approval_request_round_trips_through_json() {
        let block = ContentBlock::ApprovalRequest {
            id: uuid::Uuid::nil(),
            tool_name: "delete_file".to_string(),
            description: "Delete src/old.js".to_string(),
            input: serde_json::json!({"path": "src/old.js"}),
            status: ApprovalStatus::Pending,
            decided_at: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        let parsed: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(block, parsed);
    }
}
