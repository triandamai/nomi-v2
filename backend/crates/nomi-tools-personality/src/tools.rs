//! The personality tools as `Tool`s: what `PersonalityAgent` runs, and what custom agents can be
//! granted.

use nomi_tool::ToolContext;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

use nomi_tool::{Tool, ToolOutcome};
use nomi_llm::ToolDefinition;

use crate::PERSONALITY_AGENT_TYPE;

/// Every personality tool, in the order the agent is offered them. All of them run for any agent.
pub fn tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(SetPersonality), Box::new(ResetChatPersonality), Box::new(ListPersonalityVersions), Box::new(RollbackPersonality)]
}

pub struct SetPersonality;

#[async_trait]
impl Tool for SetPersonality {
    fn definition(&self) -> ToolDefinition {
        crate::set_personality_tool_definition()
    }
    fn group(&self) -> &'static str {
        PERSONALITY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["tone", "style", "voice", "behavior"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (session_id, agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::set_personality(conn, session_id, agent_session_id, user_id, input).await.map(ToolOutcome::text)
    }
}

pub struct ResetChatPersonality;

#[async_trait]
impl Tool for ResetChatPersonality {
    fn definition(&self) -> ToolDefinition {
        crate::reset_chat_personality_tool_definition()
    }
    fn group(&self) -> &'static str {
        PERSONALITY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["default", "tone", "clear", "chat"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, _input: Value) -> Result<ToolOutcome, String> {
        let (session_id, _agent_session_id, _user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::reset_chat_personality(conn, session_id).await.map(ToolOutcome::text)
    }
}

pub struct ListPersonalityVersions;

#[async_trait]
impl Tool for ListPersonalityVersions {
    fn definition(&self) -> ToolDefinition {
        crate::list_personality_versions_tool_definition()
    }
    fn group(&self) -> &'static str {
        PERSONALITY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["history", "tone", "past", "style"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, _input: Value) -> Result<ToolOutcome, String> {
        let (session_id, _agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::list_personality_versions(conn, session_id, user_id).await.map(ToolOutcome::text)
    }
}

pub struct RollbackPersonality;

#[async_trait]
impl Tool for RollbackPersonality {
    fn definition(&self) -> ToolDefinition {
        crate::rollback_personality_tool_definition()
    }
    fn group(&self) -> &'static str {
        PERSONALITY_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["restore", "undo", "version", "tone"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (session_id, agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::rollback_personality(conn, session_id, agent_session_id, user_id, input).await.map(ToolOutcome::text)
    }
}
