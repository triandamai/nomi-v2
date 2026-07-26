# Sub-Agent Spawning: Tool-Calling Loop, State Machine Wiring & Money Agent

Date: 2026-07-26
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`
Related: `docs/superpowers/specs/2026-07-22-sub-agent-lifecycle-design.md` (the `agent_sessions` state machine and advisory-lock concurrency this plan wires into the turn loop — already fully designed, not re-litigated here), `docs/superpowers/specs/2026-07-26-orchestrator-turn-loop-design.md` (the turn loop this plan extends — `routing.rs`'s always-`None` stub gets real logic here), `docs/superpowers/specs/2026-07-26-llm-provider-abstraction-design.md` (the `LlmProvider`/`ToolDefinition`/`ContentBlock::{ToolUse,ToolResult}` shapes this plan's tool-calling loop drives)

## Purpose

The parent plan describes the orchestrator as continuing an active sub-agent, spawning a new one, or answering as chitchat — but until now, only chitchat has been built; `routing::find_active_agent_session` exists and is tested, but nothing ever populates `agent_sessions`, so every turn falls through to chitchat unconditionally. This document designs: (1) a generic, agent-agnostic multi-turn tool-calling loop (nothing in the codebase yet executes a tool the LLM asks for — `LlmProvider` only does one-shot completions), (2) wiring the already-designed `agent_sessions` state machine (spawn/continue/complete/cancel/expire) into the turn loop in place of the stub, and (3) one concrete, working sub-agent — the money agent — to prove the whole mechanism end-to-end, not just its pieces in isolation.

Scoped down deliberately: no booking agent (a second, structurally similar agent, deferred to its own future plan once this pattern is proven), no real bank/PSP integration (the money agent's data source is a stub table), no human-approval-event step (irrelevant since this agent never executes anything, only advises), no live MQTT (agent_events remains the durable audit record, as in every prior plan).

## 1. Architecture

Three new modules under `backend/src/turn/`, plus real logic in the existing `routing.rs` stub:

- **`tools.rs`** — the generic tool-calling loop. Given a system prompt, an agent's tool registry, and conversation history, it repeatedly calls `provider.complete(...)` and, on `StopReason::ToolUse`, executes each `ToolUse` block via the registry and feeds a `ToolResult` back into the next request — until `StopReason::EndTurn` (agent pauses, waiting for the user's next message), the shared `complete_task` tool is called (agent is done), or a hard cap of 10 tool-turns is hit (`TurnError::ToolLoopExceeded`).
- **`subagent.rs`** — the `SubAgent` trait every concrete sub-agent implements: `agent_type() -> &'static str`, `system_prompt() -> &'static str`, `tools() -> Vec<ToolDefinition>` (agent-specific tools only — the loop always adds `complete_task` on top), `execute_tool(name, input, ctx) -> Result<String, ToolError>`. `complete_task` calls are intercepted by the loop itself, never delegated to `execute_tool`, since marking `agent_sessions` completed/cancelled is universal, not agent-specific.
- **`money_agent.rs`** — the concrete `MoneyAgent`: `list_transactions`, `summarize_budget` tools (both read-only, backed by a new `mock_transactions` table), plus a system prompt establishing the advisory-only, read/categorize/summarize-only framing from the parent plan.
- **`routing.rs`** (existing file, `find_active_agent_session` unchanged) gains new functions for intent classification, spawning, and lazy expiry — replacing `handle_inbound_message`'s current "always falls through to chitchat" behavior.

`ToolContext { pool: PgPool, user_id: Uuid }` is threaded into every tool execution — enough for `MoneyAgent`'s DB-backed tools today, generic enough for a future agent's tools without needing a trait change.

## 2. State Machine Wiring

Inside `handle_inbound_message`, after Txn A (inbound message durable) and the existing `routing::find_active_agent_session` call:

1. **Active agent found**: check `last_activity_at` against the lifecycle design's 24h timeout.
   - **Not stale**: continue — load the row's `agent_type`, and reconstruct conversation context from the same last-20-session-messages fetch chitchat already uses (the sub-agent's own prior replies and the user's messages since it spawned are just ordinary rows in `messages`, exactly like chitchat's). The persisted `state` JSONB is *not* a substitute for that history — it exists solely for whatever agent-specific scratch data doesn't naturally live as a chat message (e.g. a future booking agent tracking "already collected: date, time; still need: party size"). For the money agent in this plan, which only runs read-only queries per turn, `state` stays `'{}'` throughout — it's plumbed through now so the mechanism exists, not because this agent needs it.
   - **Stale**: mark it `expired` (+ `AgentExpired` event), fall through to step 2 as if nothing was active.
2. **No active agent found**: classify intent via a cheap LLM call, reusing the same `provider` already passed into `handle_inbound_message` (same reuse pattern as the RAG plan's extraction step) — result is exactly `chitchat` or `money`.
   - **`chitchat`**: unchanged — `run_chitchat_turn` runs exactly as it does today.
   - **`money`**: spawn — insert a new `agent_sessions` row (`status = 'active'`, `agent_type = 'money'`, `state = '{}'`) + an `AgentSpawned` event, then immediately run the money agent's first turn via the same tool-calling loop as "continue."
3. **After the loop returns**: a `Reply(String)` (via `EndTurn`) persists the reply message and bumps `last_activity_at` — the agent stays `active`, waiting for the next inbound message. A `Completed { status, summary }` (via `complete_task`) updates the `agent_sessions` row (`status`, `ended_at`) and publishes `AgentCompleted`/`AgentCancelled` — the next inbound message for this session finds no active agent and re-enters at step 2.

## 3. Money Agent

New migration, `backend/migrations/0009_mock_transactions.sql`:

```sql
CREATE TABLE mock_transactions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id),
    occurred_at  TIMESTAMPTZ NOT NULL,
    amount_cents BIGINT NOT NULL,
    category     TEXT NOT NULL,
    description  TEXT NOT NULL
);
```

No seed data in the migration itself (matching how `memory_items` was left empty by its own migration) — tests seed their own rows; how real transaction data eventually gets in here is a future concern, explicitly out of scope.

`MoneyAgent`'s tools:
- **`list_transactions(limit: u32, category: Option<String>)`** — the user's most recent transactions (optionally filtered by category), formatted as text.
- **`summarize_budget(since: Option<String>)`** — totals grouped by category since a given date (or all-time if omitted).
- **`complete_task(status: "completed" | "cancelled", summary: String)`** — shared; handled by the loop, not `MoneyAgent::execute_tool`.

System prompt establishes explicitly: read/categorize/summarize only, no ability to move money, matching the parent plan's guardrail verbatim ("the tool registry for this agent contains things like list transactions, categorize spending, summarize budget, nothing that moves funds").

## 4. Transaction & Audit Strategy

- Every tool call (name, input, result, `is_error`) is logged as a `ToolCalled` `agent_events` row **immediately as it happens** — auto-committed, not batched with the turn's final persist step — so a loop that hits the 10-turn cap and errors out still leaves a durable record of every step attempted. This satisfies the parent plan's explicit requirement that the money agent's every recommendation is logged even though nothing executes.
- The turn's final step (persisting the reply message, updating `agent_sessions` status/state/`last_activity_at`) is one transaction, mirroring chitchat's existing Txn B shape.
- `AgentSpawned`/`AgentCompleted`/`AgentCancelled`/`AgentExpired` are their own `agent_events` rows at the relevant transitions. No MQTT — deferred everywhere in this project so far.

## 5. Error Handling

- **Intent classification failure**: falls back to chitchat (fail-open, same philosophy as the RAG plan's retrieval-failure handling) — a broken classifier must not block the user from getting *some* reply.
- **Tool execution failure** (e.g. a DB error inside `list_transactions`): the tool's `ToolResult` carries `is_error: true` and the failure message as content — the LLM sees it and can react (apologize, retry a different tool, or call `complete_task` with `cancelled`) rather than the whole turn failing. Still logged via `ToolCalled` with the error captured.
- **Loop exhausted** (`complete_task` never called, `EndTurn` never reached, 10-turn cap hit): `TurnError::ToolLoopExceeded`. The turn fails like any other `TurnError` — inbound message stays durable via Txn A, a `TurnFailed` event is recorded, the lock is released, and the `agent_sessions` row is left exactly as it was (not marked completed, since it didn't complete). The user's next message re-enters at the "continue" branch and can retry.
- **LLM call itself fails** (network/provider error) at any point in the loop: `TurnError::LlmCallFailed`, same as today's chitchat failure path — turn fails, agent session state untouched.

## 6. Testing Approach

- Tool-calling loop: a hand-written fake `LlmProvider` scripted to return a sequence of responses (tool-use → tool-use → `complete_task`, or tool-use → `EndTurn`, or tool-use × 11 to force the cap) proves the loop executes tools, feeds results back correctly, and terminates on each of the three exit conditions.
- `MoneyAgent`'s tools: `#[sqlx::test]` seeding `mock_transactions`, asserting `list_transactions`/`summarize_budget` return correct, correctly-`user_id`-scoped results, and that a tool error (e.g. malformed input) produces an `is_error: true` result rather than a panic.
- State machine wiring: `#[sqlx::test]` covering spawn (new `agent_sessions` row + `AgentSpawned` event), continue (existing active row reused, no duplicate spawn), complete (via `complete_task`, row transitions to `completed`/`cancelled` with `ended_at` set), expire (a stale active row plus a new inbound message routes to chitchat, old row transitions to `expired`).
- Intent classification: fake `LlmProvider` returning `"money"`/`"chitchat"`/garbage text, proving correct parsing and the fail-open-to-chitchat behavior on a malformed or failed classification.
- End-to-end: a full `handle_inbound_message` test simulating a multi-turn money-agent conversation — spawn → tool calls → reply → a second inbound message continues the same agent (no re-spawn, no re-classification) → `complete_task` → agent marked completed → a third inbound message routes to chitchat again (proving the state machine's full cycle, not just its individual transitions).

## Out of Scope

- The booking agent — deferred to its own future plan once this pattern is proven with money.
- Real bank/PSP API integration — `mock_transactions` is a stub table, not a real data source.
- The human-approval-event step (`ActionPendingApproval` → `ActionApproved`) — irrelevant since the money agent never executes anything, only advises.
- Live MQTT publishing — `agent_events` remains the durable audit record, as in every prior plan in this project.
