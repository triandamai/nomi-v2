//! The background-task tools as `Tool`s: every agent gets them, they run straight away, and they
//! post nothing for an agent's activity (the task card is the message).

use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use nomi_tool::{Approval, AutoOffer, Tool, ToolContext, ToolOutcome};
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

/// The catalog area these tools are listed under.
pub const GROUP: &str = "background";

/// `run_background_task`, `list_background_tasks` and `cancel_background_task`, in that order.
pub fn tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(RunBackgroundTask), Box::new(ListBackgroundTasks), Box::new(CancelBackgroundTask)]
}

pub struct RunBackgroundTask;
pub struct ListBackgroundTasks;
pub struct CancelBackgroundTask;

macro_rules! background_properties {
    () => {
        fn group(&self) -> &'static str {
            GROUP
        }
        fn approval(&self) -> Approval {
            Approval::Never
        }
        fn auto_offer(&self) -> AutoOffer {
            AutoOffer::Everyone
        }
        fn posts_activity(&self) -> bool {
            false
        }
    };
}

#[async_trait]
impl Tool for RunBackgroundTask {
    fn definition(&self) -> ToolDefinition {
        crate::run_background_task_tool_definition()
    }
    background_properties!();
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (display_text, card) = crate::start_background_task(conn, ctx.mqtt, ctx.session_id, ctx.user_id, ctx.agent_type, &input).await?;
        Ok(ToolOutcome { display_text, block: Some(card) })
    }
}

#[async_trait]
impl Tool for ListBackgroundTasks {
    fn definition(&self) -> ToolDefinition {
        crate::list_background_tasks_tool_definition()
    }
    background_properties!();
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, _input: Value) -> Result<ToolOutcome, String> {
        crate::list_background_tasks(conn, ctx.session_id, ctx.user_id).await.map(ToolOutcome::text)
    }
}

#[async_trait]
impl Tool for CancelBackgroundTask {
    fn definition(&self) -> ToolDefinition {
        crate::cancel_background_task_tool_definition()
    }
    background_properties!();
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        crate::cancel_background_task(conn, ctx.mqtt.map(|(p, _)| p), ctx.session_id, ctx.user_id, &input).await.map(ToolOutcome::text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_tools_properties() {
        let tools = tools();
        let names: Vec<String> = tools.iter().map(|t| t.definition().name).collect();
        assert_eq!(names, crate::BACKGROUND_TOOL_NAMES);
        for t in &tools {
            assert_eq!(t.approval(), Approval::Never);
            assert_eq!(t.auto_offer(), AutoOffer::Everyone);
            assert!(!t.posts_activity());
        }
    }
}
