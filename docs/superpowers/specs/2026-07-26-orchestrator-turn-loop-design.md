# Orchestrator Turn Loop & Chitchat Agent

Date: 2026-07-26
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`
Related: `docs/superpowers/specs/2026-07-22-sub-agent-lifecycle-design.md`, `docs/superpowers/specs/2026-07-22-cross-channel-identity-design.md`, `docs/superpowers/specs/2026-07-25-multi-tenant-rbac-design.md`, `docs/superpowers/specs/2026-07-26-llm-provider-abstraction-design.md` (this doc consumes that trait)

## Purpose

The backend has a schema (sessions, messages, identity, org, sub-agent lifecycle tables) and an auth/claims layer, but no application logic that actually processes a message yet. This document designs the turn-loop skeleton — advisory lock, identity/org bootstrap, the two-transaction split, and the (currently trivial) sub-agent routing check — wired up to a real chitchat agent using the LLM provider abstraction. Explicitly scoped down from the full "turn loop" concept: no live MQTT publishing, no RAG/memory retrieval, no real sub-agent spawning, and no real channel adapter (Telegram/WhatsApp) — those are separate future plans. This slice is testable end-to-end via a plain function call simulating what a channel adapter will eventually provide.

## 1. Entry Point

```rust
pub async fn handle_inbound_message(
    pool: &PgPool,
    provider: &dyn LlmProvider,
    channel: &str,          // "telegram" | "whatsapp" | "web" | ...
    chat_type: &str,        // "dm" | "group"
    chat_id: &str,          // platform's chat/group id
    sender_channel_user_id: &str,
    text: &str,
) -> Result<TurnOutcome, TurnError>
```

No schema changes are needed for this plan — `users`, `channel_identities`, `organizations`, `memberships`, `sessions`, `messages`, `agent_sessions`, and `agent_events` all already exist from the schema-migrations and auth-claims-layer plans.

## 2. Steps

1. **Identity + org bootstrap**: look up `channel_identities` by `(channel, sender_channel_user_id)`. If not found: in one transaction, create a `users` row, an auto-created personal `organizations` row (`is_personal = true`), a `memberships` row (`role = 'owner'`) for that user in that org, and the `channel_identities` row pointing at the new user — per the multi-tenant-rbac design's §3 ("the first time any user is bootstrapped, an organizations row is auto-created"). **This closes a real gap**: that bootstrap logic was specified but never actually built by the auth-claims-layer plan (which only handled web/email registration).
2. **Session lookup/create**: look up `sessions` by `(channel, chat_id)`. If not found, create it with `org_id` = the sender's (personal, for now) org, plus `chat_type`/`chat_id`.
3. **Acquire the session-scoped advisory lock**: `SELECT pg_advisory_lock(hashtext($1))` (binding `session_id::text`) on a **single pinned connection** (`pool.acquire()`, not a fresh connection per statement) held for the entire turn — matching the sub-agent lifecycle design's explicit connection-pinning requirement, since `pg_advisory_lock` is connection-scoped, not transaction-scoped.
4. **Txn A**: insert the inbound message (`sender_channel_identity_id` = sender's identity, `content` = the message text) on that same connection, commit immediately. The message is durable regardless of what happens next.
5. **Sub-agent routing check**: query `agent_sessions` for `(session_id, sender_channel_identity_id)` where `status = 'active'`. In this slice, nothing ever populates that table, so this always returns "not found" and falls through to chitchat unconditionally — but the query itself, and the state-machine check, are implemented and tested now. A future sub-agent plan adds only the spawn branch; this scaffolding doesn't change.
6. **Txn B**: fetch the last 20 messages for the session (ordered by `created_at`), map to `LlmMessage`s (`User` when `sender_channel_identity_id IS NOT NULL`, `Assistant` when `NULL`, each a single `Text` content block), build an `LlmRequest` with a fixed chitchat system prompt and `tools: vec![]`, call `provider.complete(request)`. On success: persist the reply as a new message (`sender_channel_identity_id = NULL`), insert an `agent_events` row (`event_type = "ChitchatReply"`, JSONB payload recording `input_tokens`/`output_tokens`), commit.
7. **Release the advisory lock** — always, on both the success and failure paths (`SELECT pg_advisory_unlock($1)` on the same pinned connection).

## 3. Chitchat Prompt & History

- **History**: last 20 messages for the session, ordered oldest-first, each mapped to a single-`Text`-block `LlmMessage` as above.
- **System prompt**: a fixed constant — "You are a helpful, friendly assistant chatting with the user. Keep replies concise." No per-user customization, no RAG-retrieved memory in this slice (that's the deferred RAG piece from the earlier scoping decomposition).
- **Group chats**: every inbound message gets a chitchat reply regardless of `chat_type` in this generic engine — whether a real Telegram/WhatsApp adapter should only reply when @-mentioned in a group is that future adapter's own decision, not this turn loop's.

## 4. Error Handling

- **Txn A always succeeds independently of Txn B** — the durability guarantee from the sub-agent lifecycle design carries over unchanged.
- **If `provider.complete()` fails** (network error, non-2xx, timeout): Txn B is never committed, so no assistant reply is persisted and `agent_sessions` state (irrelevant in this slice, but the invariant holds for later) is untouched. A `TurnFailed` `agent_events` row is written best-effort, **outside** Txn B (a separate, always-attempted insert). The advisory lock is released regardless (success or failure path — implemented via an explicit release in both branches, not a scope guard, to keep the release point visible and testable). `handle_inbound_message` returns `Err(TurnError::LlmCallFailed(...))`. No automatic retry — the next inbound message for this session naturally re-enters at step 1, consistent with "a missing reply reads as the assistant not having answered yet."
- **Provider returns `stop_reason: MaxTokens`** (truncated reply): still treated as a successful turn — the truncated text is persisted as-is. Prompt/`max_tokens` tuning is future work, not a turn-loop concern.

## 5. Testing Approach

- **Turn-loop mechanics**: tested against real Postgres (`sqlx::test`) with a hand-written test `LlmProvider` (returns a fixed `LlmResponse`, or a configurable error) — no real LLM calls. Assert: identity/org/session bootstrap happens correctly on a brand-new sender; an existing sender reuses their existing identity/org/session; the inbound message is present even when the provider call fails; the assistant reply is persisted with `sender_channel_identity_id = NULL` on success; an `agent_events` row exists for both the success (`ChitchatReply`) and failure (`TurnFailed`) cases.
- **Concurrency test**: two simulated inbound messages for the same session, fired concurrently (e.g. via `tokio::join!` on two `handle_inbound_message` calls) — assert they serialize (the second's Txn A doesn't start until the first's full turn, lock included, has released), matching the sub-agent lifecycle design's own stated test requirement.
- **Provider integration**: the real Anthropic/OpenAI/Gemini HTTP mapping is tested separately, per the LLM provider abstraction design's own testing section (mocked HTTP, no live keys) — the turn-loop tests never depend on that.

## Out of Scope (deferred, not blocking this spec)

- Live MQTT event publishing (the `agent_events` row is this slice's durable audit record; a real broker integration is separate future work).
- RAG/memory retrieval (`memory_items` pgvector search) folded into the chitchat prompt.
- Real sub-agent spawning (money/booking) — the routing check exists but always falls through to chitchat.
- A real channel adapter (Telegram/WhatsApp inbound webhook + outbound send) — this slice is tested via direct function calls simulating what an adapter would pass in.
