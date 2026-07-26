# RAG Memory Retrieval, Writing & Reinforcement

Date: 2026-07-26
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`
Related: `docs/superpowers/specs/2026-07-26-orchestrator-turn-loop-design.md` (this doc extends `run_chitchat_turn`, deferred RAG explicitly out of that plan's scope), `docs/superpowers/specs/2026-07-26-llm-provider-abstraction-design.md` (the `EmbeddingProvider` trait mirrors this plan's `LlmProvider` trait shape), `docs/superpowers/specs/2026-07-22-cross-channel-identity-design.md` (memory is scoped by canonical `user_id`, not `session_id` or `channel_identity`)

## Purpose

The parent plan describes RAG as: "Text gets embedded → pgvector similarity search over `memory_items`, filtered by user, re-ranked by `weight` (the reinforcement signal) alongside raw similarity," plus a `reinforce()` feedback loop that nudges a memory's `weight` up or down based on outcome. The turn-loop plan explicitly deferred all of this ("RAG/memory retrieval (`memory_items` pgvector search) folded into the chitchat prompt" — Out of Scope). This document designs the full loop: retrieval (folding relevant memories into the chitchat prompt), writing (how a `memory_items` row gets created in the first place, via LLM-judged extraction after a successful reply), and reinforcement (a standalone `reinforce()` function nudging weight — not wired to any UI yet, since none exists).

The `memory_items` table already exists (migration `0006_memory_and_events.sql`): `id`, `user_id`, `content`, `embedding VECTOR(1536)`, `weight DOUBLE PRECISION DEFAULT 1.0`, `created_at`, `updated_at`. No changes needed to it — `VECTOR(1536)` already matches OpenAI's `text-embedding-3-small`, and `weight` already supports the reinforcement formula below.

## 1. Architecture

A new `backend/src/embedding/` module, mirroring the shape of the existing `backend/src/llm/` module:

```rust
// backend/src/embedding/mod.rs
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
}
```

`OpenAiEmbeddingProvider` (`backend/src/embedding/openai.rs`) implements it: `POST {base_url}/v1/embeddings`, model `text-embedding-3-small`, same constructor-parameter discipline as every existing provider (`api_key`/`base_url` explicit, never read from environment). A parallel `EmbeddingConfig`/`build_embedding_provider` pair (`backend/src/embedding/config.rs`) mirrors `ModelConfig`/`build_provider` from the LLM layer.

`handle_inbound_message` gains one new parameter: `embedding_provider: &dyn EmbeddingProvider`, threaded through to `run_chitchat_turn`. `run_chitchat_turn` gains two new parameters: `user_id: Uuid` (already resolved by `bootstrap_identity_and_session`, currently discarded via `..` in `handle_inbound_message` — this plan stops discarding it) and `text: &str` (the current turn's inbound message, already available in `handle_inbound_message` as its own `text` parameter — passed explicitly for embedding rather than re-derived by assuming it's the last row of the history fetch, which would be an implicit and more fragile dependency on fetch-then-insert ordering).

A new `backend/src/turn/memory.rs` owns retrieval, extraction/writing, and reinforcement. `chitchat.rs` calls into it; `memory.rs` owns the RAG-specific SQL and prompt-fragment building, keeping `chitchat.rs` focused on the conversation turn itself.

## 2. Data Model

One new migration, `backend/migrations/0008_message_memory_usage.sql`:

```sql
CREATE TABLE message_memory_usage (
    message_id  UUID NOT NULL REFERENCES messages(id),
    memory_id   UUID NOT NULL REFERENCES memory_items(id),
    PRIMARY KEY (message_id, memory_id)
);
```

Links a specific reply message to the memories that were retrieved and folded into its prompt — this is what `reinforce()` uses to find which memories to nudge for a given reply.

## 3. Retrieval Flow (inside `run_chitchat_turn`, before the chitchat LLM call)

1. Embed the incoming user message (the `text` parameter, §1) via `embedding_provider.embed(text)` — outside any DB transaction, matching the existing rationale for the chitchat LLM call (no connection/lock held across slow network I/O).
2. On success, query the top 5 memories by weighted similarity, scoped to the sender's canonical `user_id` (not `session_id` — memory is per-person, shared across all their channels/sessions, per the cross-channel-identity design):
   ```sql
   SELECT id, content, weight, 1 - (embedding <=> $1) AS similarity
   FROM memory_items
   WHERE user_id = $2
   ORDER BY weight * (1 - (embedding <=> $1)) DESC
   LIMIT 5
   ```
3. On failure (embedding call errors, or the query errors), log and proceed with an empty memory list — chitchat still replies normally, just without RAG context. This is not a turn failure.
4. Fold retrieved memories into the system prompt, appended after the fixed chitchat instructions:
   ```
   You are a helpful, friendly assistant chatting with the user. Keep replies concise.

   Relevant things you know about this user:
   - <memory 1 content>
   - <memory 2 content>
   ```
   If no memories were retrieved (empty list, whether from a genuinely empty result or a swallowed failure), omit the "Relevant things..." section entirely — never send an empty section.
5. Retain the retrieved memory `id`s for step 4 of §4 below (linking them to the reply once it's persisted).

## 4. Persisting the Reply + Memory Links (Txn B, extended)

Txn B (already existing: insert reply message, insert `ChitchatReply` event) is extended:
- The reply-message insert becomes `INSERT ... RETURNING id` (needed so `message_memory_usage` has a `message_id` to reference — today's insert doesn't return one).
- One `message_memory_usage` row is inserted per memory retrieved in §3, linking the new reply `message_id` to each retrieved `memory_id`.

Still commits atomically with the reply and the `ChitchatReply` event — if Txn B fails, none of the reply, the event, or the memory-usage links persist, consistent with today's all-or-nothing Txn B semantics.

## 5. Memory Writing Flow (after Txn B commits)

Runs as a best-effort tail step inside `run_chitchat_turn`, after the reply is already durable — its failure never surfaces as a turn failure, since the user already has their reply by this point:

1. Call the LLM (the same `provider: &dyn LlmProvider` already passed into `run_chitchat_turn` for the chitchat reply itself) with a fixed extraction system prompt: `"Extract at most one durable fact worth remembering long-term from this exchange, or say NONE if nothing is worth storing."` — given just the user's message and the assistant's reply as the conversation.
2. If the extracted result is the literal string `NONE` (or the LLM call itself fails), stop — nothing is stored.
3. Otherwise, embed the extracted fact via `embedding_provider.embed(...)` and insert a new `memory_items` row: `user_id`, `content = <extracted fact>`, `embedding`, `weight` defaults to `1.0`.
4. Any failure anywhere in steps 1–3 (LLM call, embedding call, or the insert) is logged and ignored. `run_chitchat_turn` still returns the reply text it already produced in §3–4 — this step never changes the turn's outcome.

## 6. Reinforcement

A standalone public function in `memory.rs`. Not wired into `handle_inbound_message` or any other caller in this plan — there is no UI or channel command yet to trigger it; that wiring is future work.

```rust
pub enum ReinforcementSignal { Positive, Negative }

#[derive(Debug, thiserror::Error)]
pub enum ReinforcementError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub async fn reinforce(
    pool: &PgPool,
    reply_message_id: Uuid,
    signal: ReinforcementSignal,
) -> Result<(), ReinforcementError> {
    let factor = match signal {
        ReinforcementSignal::Positive => 1.2,
        ReinforcementSignal::Negative => 0.8,
    };
    // UPDATE memory_items SET weight = LEAST(GREATEST(weight * $1, 0.1), 5.0)
    // WHERE id IN (SELECT memory_id FROM message_memory_usage WHERE message_id = $2)
}
```

Weight is clamped to `[0.1, 5.0]`: never reaches `0` (which would make a memory permanently unreachable regardless of similarity), and never grows unbounded from repeated positive reinforcement. If `reply_message_id` has no `message_memory_usage` rows (no memories were used for that reply, or the id doesn't correspond to a reply at all), `reinforce` is a no-op, not an error.

## 7. Error Handling Summary

- **Retrieval failure** (embedding call or similarity query errors): logged, chitchat proceeds without memory context. Turn still succeeds — same as if no relevant memories existed.
- **Chitchat LLM failure**: unchanged from the turn-loop plan — turn fails, `TurnFailed` event, inbound message stays durable via Txn A.
- **Txn B failure**: unchanged — no reply, no `ChitchatReply` event, no `message_memory_usage` links persist.
- **Extraction/embedding/write failure** (post-reply, §5): logged, ignored, turn still returns `Ok` with the reply already produced.

## 8. Testing Approach

- `EmbeddingProvider`/`OpenAiEmbeddingProvider`: same pattern as the existing LLM providers — `wiremock`-based tests asserting the request/response shape, no real network calls, no real API keys.
- Retrieval: `#[sqlx::test]` seeding `memory_items` with known embeddings/weights across multiple users, asserting the top-5 weighted-similarity ranking and that a different user's memory is never retrieved.
- Writing: a hand-written fake `LlmProvider` + fake `EmbeddingProvider` pair — asserting a `NONE` extraction stores nothing, and a real extracted fact gets embedded, stored, and linked via `message_memory_usage` to the correct reply message.
- Reinforcement: `#[sqlx::test]` asserting `Positive`/`Negative` signals nudge weight in the right direction and stay clamped to `[0.1, 5.0]` under repeated calls; asserting a `reply_message_id` with no memory usage is a no-op.
- Graceful degradation: a failing embedding/extraction fake still lets the overall turn succeed end-to-end (reply returned, message persisted, no memory-related error surfaced) with the RAG step silently skipped — mirrors the turn-loop plan's own "provider failure durability" test pattern.

## Out of Scope

- Any UI or channel-command surface that actually calls `reinforce()` — this plan only builds the function itself.
- Memory summarization, consolidation, decay over time, or pruning of low-weight/stale memories.
- Cross-session memory sharing beyond the existing per-`user_id` scoping (already correct by construction — no new work needed here).
- Embedding providers other than OpenAI (the trait allows it; only one implementation is built now, matching how the LLM abstraction plan built all three provider implementations up front but this plan builds only one — embeddings have no equivalent "pick a model per agent" requirement driving multi-provider support yet).
