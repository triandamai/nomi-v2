//! The health agent. Its tools, and the client they run through, live in `nomi-tools-health`,
//! re-exported here.

use std::borrow::Cow;
use std::ops::Deref;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::content_block::ToolOutcome;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub use nomi_tools_health::*;

const SYSTEM_PROMPT: &str = "You are Raga, the crew member for the person's health and training. \
You work in their own Hevy account (workouts, routines, routine folders, exercises, exercise \
history, body measurements). You only ever see the account of the person you are helping.

How you work:
- Exercises are referenced by their Hevy exercise template id. Find ids with hevy_find_exercises \
before logging a workout or building a routine; never invent ids. If an exercise doesn't exist, \
offer to create a custom one.
- Logging a workout: ask only for what's missing (which exercises, sets, reps, weight), with \
ask_user, all of it in one card. Times \
are ISO 8601 with the person's offset; if they say \"today\" or \"this morning\", work it out \
from the current time. Weights are in kg (convert from lb when they use lb).
- Updating a workout or routine replaces it whole: read it first, change what was asked, send \
everything back.
- Progress questions (\"how is my bench going?\"): read the exercise history and answer with \
the trend in a sentence or two (best set, volume, how often), then the numbers.
- Show workouts, routines or measurements with `show` when there are more than a few: a list or a table, \
a chart for progress over time.
- Keep advice practical and safe. You aren't a doctor: for pain, injury or medical questions, \
suggest seeing a professional.
- Heart rate, blood oxygen, steps, sleep and runs from Samsung Health or Apple Health aren't \
connected yet; say so if asked.
- If a tool says Hevy isn't connected or the key stopped working, a card to set it up is \
already shown to the person: tell them in one short sentence and stop.";

/// The crew member, running every tool through its `HealthClient`.
#[derive(Clone)]
pub struct HealthAgent {
    client: HealthClient,
}

impl HealthAgent {
    pub fn new(http: reqwest::Client, key: [u8; 32]) -> Self {
        Self::from_client(HealthClient::new(http, key))
    }

    pub fn with_hevy_base_url(self, base_url: impl Into<String>) -> Self {
        Self::from_client(self.client.with_hevy_base_url(base_url))
    }

    pub fn from_client(client: HealthClient) -> Self {
        Self { client }
    }

    pub fn from_env() -> Self {
        Self::from_client(HealthClient::from_env())
    }

    pub fn client(&self) -> &HealthClient {
        &self.client
    }
}

impl Deref for HealthAgent {
    type Target = HealthClient;
    fn deref(&self) -> &HealthClient {
        &self.client
    }
}

/// Every health tool, run through `agent`'s client, for the catalog.
pub fn catalog_tools(agent: &HealthAgent) -> Vec<Box<dyn nomi_agent_core::Tool>> {
    nomi_tools_health::tools(&agent.client)
}

#[async_trait]
impl SubAgent for HealthAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(HEALTH_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(SYSTEM_PROMPT)
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("health")
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("health, fitness and training: logging or reviewing workouts, gym sessions and exercises, building or changing workout routines, progress on a lift, body weight and measurements, the person's Hevy account")
    }

    fn uses_memory(&self) -> bool {
        true
    }

    fn uses_personality(&self) -> bool {
        true
    }

    fn wants_current_time(&self) -> bool {
        true
    }

    fn max_tool_turns(&self) -> u32 {
        16
    }

    /// Everything here is the person's own training log in their own Hevy account: nothing
    /// reaches anyone else, so nothing waits for an OK.
    fn tool_needs_approval(&self, _tool_name: &str) -> bool {
        false
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        tool_definitions()
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<ToolOutcome, String> {
        self.run(conn, user_id, name, input).await
    }
}
