//! The crew's scheduled-job tools as `Tool`s: offered to agents that take reminders, run
//! straight away, and post nothing for an agent's activity.

use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use nomi_tool::{Approval, AutoOffer, OfferContext, Tool, ToolContext, ToolOutcome};
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

/// The catalog area these tools are listed under (Kala's own tools share the area, but are only
/// offered on a grant).
pub const GROUP: &str = "reminders";

/// `create_reminder`, `list_reminders` and `cancel_reminder`, in that order.
pub fn tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(CreateReminder), Box::new(ListReminders), Box::new(CancelReminder)]
}

macro_rules! reminder_properties {
    () => {
        fn group(&self) -> &'static str {
            GROUP
        }
        fn approval(&self) -> Approval {
            Approval::Never
        }
        fn auto_offer(&self) -> AutoOffer {
            AutoOffer::IfSupportsReminders
        }
        fn posts_activity(&self) -> bool {
            false
        }
    };
}

pub struct CreateReminder;

#[async_trait]
impl Tool for CreateReminder {
    /// Without a conversation there are no agents to target; the offer lists them.
    fn definition(&self) -> ToolDefinition {
        super::create_reminder_tool_definition(&[])
    }
    fn offer(&self, ctx: &OfferContext<'_>) -> Option<ToolDefinition> {
        Some(super::create_reminder_tool_definition(ctx.reminder_targets))
    }
    reminder_properties!();
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        super::create_reminder(conn, ctx.session_id, ctx.user_id, ctx.agent_type, &input).await.map(ToolOutcome::text)
    }
}

pub struct ListReminders;

#[async_trait]
impl Tool for ListReminders {
    fn definition(&self) -> ToolDefinition {
        super::list_reminders_tool_definition()
    }
    reminder_properties!();
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, _input: Value) -> Result<ToolOutcome, String> {
        super::list_reminders(conn, ctx.user_id).await.map(ToolOutcome::text)
    }
}

pub struct CancelReminder;

#[async_trait]
impl Tool for CancelReminder {
    fn definition(&self) -> ToolDefinition {
        super::cancel_reminder_tool_definition()
    }
    reminder_properties!();
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        super::cancel_reminder(conn, ctx.user_id, &input).await.map(ToolOutcome::text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_job_tools_properties() {
        let tools = tools();
        let names: Vec<String> = tools.iter().map(|t| t.definition().name).collect();
        assert_eq!(names, [super::super::CREATE_REMINDER_TOOL_NAME, super::super::LIST_REMINDERS_TOOL_NAME, super::super::CANCEL_REMINDER_TOOL_NAME]);
        for t in &tools {
            assert_eq!(t.approval(), Approval::Never);
            assert_eq!(t.auto_offer(), AutoOffer::IfSupportsReminders);
            assert!(!t.posts_activity());
        }
    }

    #[test]
    fn create_reminder_lists_the_targets_it_is_offered_with() {
        let targets = vec!["dana".to_string()];
        let offered = CreateReminder.offer(&OfferContext { reminder_targets: &targets, mentions_files: false }).unwrap();
        assert!(offered.input_schema.to_string().contains("\"dana\""));
    }
}
