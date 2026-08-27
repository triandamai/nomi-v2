# Personality Customization (with Audit Trail and Cross-Chat Identity)

Date: 2026-08-28
Status: Approved (pending user review of this doc)
Related: `docs/superpowers/specs/2026-08-26-backend-cargo-workspace-design.md` (the `SubAgent`/`AgentRegistry` architecture this design's `PersonalityAgent` is the second real exercise of, after `MoneyAgent`).

## Purpose

A user should be able to ask nomi, in the course of normal conversation, to change how it talks or behaves ("be more sarcastic", "talk like a 1920s detective", "be more formal from now on"), have that take effect on every subsequent reply, and have every change recorded as an auditable event. That personality should follow the person, not the chat window: if the same person talks to nomi in more than one chat context (their personal DM and a group chat, or simply two separate sessions in the web UI's sidebar), nomi's personality — and its memory of them — should be consistent, because both are properties of *who they are*, not *which conversation this is*.

## 1. Cross-Chat Identity — Already Solved, Just Needs to Be Used Correctly

Before designing anything new: the codebase already unifies a person's identity across chat contexts. `channel_identities` resolves `(channel, channel_user_id)` to one stable `user_id` regardless of `chat_type` (`"dm"` vs `"group"`) or `chat_id` — proven by the existing test `bootstrap_identity_and_session`'s `existing_sender_new_chat_creates_a_new_session_under_the_same_personal_org`, which asserts the same sender gets the same `user_id` across a `"dm"` session and a separate `"group"` session. `memory_items` (long-term facts nomi has learned) is already keyed by `user_id`, not `session_id`, so it already crosses that boundary correctly today.

The only requirement this design adds is: **store personality by `user_id`, the same way memory already is.** No new identity-resolution logic is needed — it falls out of the existing schema for free. This is verified with a real test in §5, not just asserted.

## 2. `PersonalityAgent` — a Second `SubAgent`

A new crate, `nomi-agent-personality`, following `nomi-agent-money`'s exact shape (spec: `docs/superpowers/specs/2026-08-26-backend-cargo-workspace-design.md` §3):

```rust
pub const PERSONALITY_AGENT_TYPE: &str = "personality";

pub struct PersonalityAgent;

impl SubAgent for PersonalityAgent {
    fn agent_type(&self) -> &'static str { PERSONALITY_AGENT_TYPE }
    fn system_prompt(&self) -> &'static str { /* see below */ }
    fn tools(&self) -> Vec<ToolDefinition> { vec![set_personality_tool()] }
    async fn execute_tool(&self, conn, user_id, name, input) -> Result<String, String> { /* see §3 */ }
    fn intent_label(&self) -> &'static str { "personality" }
    fn intent_description(&self) -> &'static str {
        "The user wants to change how nomi talks or behaves — its tone, style, or personality"
    }
    fn uses_personality(&self) -> bool { true } // see §4 — its own confirmation reply stays in-character
}
```

**System prompt:** "You help the user customize nomi's personality — the tone, style, and manner nomi should adopt in future replies. When the user describes how they want nomi to talk or behave, call `set_personality` with a concise (one or two sentence) description of that personality, written in the second person as an instruction (e.g. 'Be sarcastic and blunt, never overly polite.'). Confirm the change back to the user in a friendly way, in the *new* personality if one was just set. When you're done, call `complete_task`."

The "concise, one or two sentence" guidance is prompt-level only — there is no code-enforced length limit or validation on `description` beyond non-empty (§ Error Handling), consistent with the "no extra guardrails for this iteration" decision.

**The `set_personality` tool:**
```json
{
  "name": "set_personality",
  "description": "Set nomi's personality — how it should talk and behave in future replies — for this user.",
  "input_schema": {
    "type": "object",
    "properties": { "description": { "type": "string", "description": "A concise instruction describing the personality, e.g. 'Be sarcastic and blunt.'" } },
    "required": ["description"]
  }
}
```

This agent behaves exactly like `MoneyAgent` in the orchestrator's eyes: classified via `routing::classify_intent` alongside `"money"` and the default `"chitchat"`; when picked, `run_locked_turn` spawns a real `agent_sessions` row for it (unlike chitchat, which never gets one — see the existing `is_default()` special-casing in `nomi-turn`); it runs the standard tool-calling loop via `run_agent_turn`; when it calls `complete_task`, the `agent_sessions` row closes and the next message reclassifies fresh, most likely falling through to chitchat.

## 3. Data Model

Two additions, no changes to existing tables:

```sql
CREATE TABLE user_personality (
    user_id     UUID PRIMARY KEY REFERENCES users(id),
    description TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Holds only the *current* personality per user — a single row, upserted on every change, so every chitchat/personality turn can fetch it with a single indexed point lookup rather than scanning history. No default row exists until a user's first personality change; absence means "no personality set, use the agent's own default tone."

**Audit trail:** no new table. `agent_events` already exists exactly for this ("what did an agent do, when, for whom") and already has the columns needed (`session_id`, `agent_session_id`, `agent_type`, `event_type`, `payload`, `created_at`) — the same pattern `AgentReplied`/`ToolCalled`/`AgentCompleted` already use.

**Single access point, in `nomi-agent-core`:** both the read (§4, used by the engine on every turn) and the write (this tool) go through one new module, `nomi_agent_core::personality`, symmetric with the existing `nomi_agent_core::memory` module — the engine and every agent crate already depend on `nomi-agent-core`, never the reverse, so this keeps `user_personality`'s access in one place rather than duplicating the upsert SQL inside `nomi-agent-personality` itself:

```rust
// nomi-agent-core::personality
pub async fn get_current_personality(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Option<String>;

pub async fn set_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    new_description: &str,
) -> Result<(), TurnError>;
// Reads the current description first (for the audit payload's "old" side, None on a user's
// first-ever change), upserts user_personality, then inserts the PersonalityChanged
// agent_events row with {old_description, new_description} — both in one transaction.
```

`PersonalityAgent::execute_tool` (in `nomi-agent-personality`) calls `nomi_agent_core::personality::set_personality` directly rather than writing its own SQL. Every change — including a user changing their mind five times in one conversation — gets its own `PersonalityChanged` row, giving a complete history queryable by `user_id` across `agent_events` even though `user_personality` itself only tracks current state.

## 4. Folding Personality into Replies

`SubAgent` gains one new opt-in method, mirroring `uses_memory()` exactly (spec: workspace-split design §3):

```rust
fn uses_personality(&self) -> bool { false } // default, unchanged for MoneyAgent
```

`nomi_agent_core::engine::run_agent_turn` (already the single place every agent's system prompt gets assembled) gains one more fold-in step, parallel to how it already folds in retrieved memories:

```rust
let system_prompt = /* existing memory fold-in, unchanged */;
let system_prompt = if agent.uses_personality() {
    match crate::personality::get_current_personality(conn, user_id).await {
        Some(p) => format!("{system_prompt}\n\nAdopt this personality in your replies: {p}"),
        None => system_prompt,
    }
} else {
    system_prompt
};
```

`ChitchatAgent::uses_personality()` returns `true` — this is the actual point of the whole feature: once a user has set a personality, every future chitchat reply (in any chat context, per §1) picks it up automatically, with zero per-turn wiring beyond this one flag. `PersonalityAgent::uses_personality()` also returns `true`, purely so its own confirmation message ("Alright, consider me thoroughly unimpressed from now on.") is delivered in-character immediately, rather than the user having to wait one more turn to see the new tone.

`MoneyAgent` does not opt in — a budget summary should stay plainly legible regardless of personality; this is a per-agent decision by design, not a global switch.

## 5. Testing

**Backend** (`#[sqlx::test]`, matching every other crate in this workspace):
- `nomi-agent-personality`: unit tests for `PersonalityAgent`'s tool definition and `execute_tool` — a `set_personality` call upserts `user_personality` and inserts a `PersonalityChanged` `agent_events` row with the correct old/new payload; a second call updates the same row (not a second one) and records `old_description` as the first value.
- `nomi-agent-core`: a `run_agent_turn` test proving `uses_personality()` folds a stored personality into the system prompt exactly once, and a control test proving an agent with `uses_personality() == false` never queries `user_personality` at all.
- `nomi-turn`: an end-to-end test (matching the shape of the `reinforce()` fix's own `a_chitchat_replys_used_memory_is_linked_and_recorded_as_an_agent_replied_event` test) driving two full `handle_inbound_message` calls through the real `AgentRegistry` — first classified as `"personality"` and calling `set_personality`, second (a plain chitchat message) asserting the system prompt sent to the LLM contains the personality text.
- **The cross-chat-identity proof**, the one genuinely new integration test this feature needs: seed a personality change via one `(channel, chat_type="dm", chat_id)`, then send a plain chitchat message through a *different* `chat_id` with `chat_type="group"` but the *same* `channel_user_id`, and assert the second call's system prompt still contains the personality set in the first. This is the concrete verification that §1's claim ("already solved, just needs to be used correctly") actually holds for this feature, not an assumption.

## Error Handling

| Condition | Behavior |
|---|---|
| `set_personality` called with an empty or missing `description` | Tool returns an error string (surfaced to the LLM, which should re-ask the user), matching how `MoneyAgent`'s tools already report bad input — no special-casing needed, the existing tool-calling loop's error path handles this. |
| User has never set a personality | `uses_personality()`'s fold-in is a no-op (§4); chitchat behaves exactly as it does today. |
| Personality fold-in query fails (DB error) | Matches the existing convention for the rest of `run_agent_turn`'s DB-backed fold-ins (memory retrieval) — degrade gracefully, log, continue with the un-folded prompt, never fail the whole turn over an enrichment step. |

## Out of Scope

- Any moderation or content filtering on what personality descriptions are allowed — per direct instruction, out of scope for this iteration.
- A UI to view/reset current personality outside of asking nomi in chat (e.g. an admin or user-settings page listing personality history) — the audit trail exists in `agent_events` and is queryable, but no frontend surface is built for it here.
- Per-organization or per-session personality (e.g. "professional tone in this specific work chat, casual everywhere else") — explicitly per-user only, matching how memory already works.
- Rate-limiting how often a user can change personality.
