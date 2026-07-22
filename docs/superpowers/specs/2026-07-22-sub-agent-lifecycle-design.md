# Sub-Agent Lifecycle & Concurrency Design

Date: 2026-07-22
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`

## Purpose

The original plan states the orchestrator "continues an active sub-agent, spawns a new one, or answers as chitchat," but leaves three things undefined:

1. How the router knows a sub-agent is "active" and what state it's holding.
2. How concurrent messages for the same session are serialized so turns never interleave or corrupt state.
3. What happens on failure mid-turn, given the plan's own durability promise that an inbound message must survive even if downstream processing fails.

This document resolves all three.

## Data model: `agent_sessions`

A new table, separate from the permanent `sessions`/`messages` thread — it holds ephemeral task state, not the permanent conversation.

```sql
CREATE TABLE agent_sessions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id                  UUID NOT NULL REFERENCES sessions(id),
    sender_channel_identity_id  UUID NOT NULL REFERENCES channel_identities(id),
    agent_type                  TEXT NOT NULL,              -- 'money' | 'booking' | future types
    status                      TEXT NOT NULL,              -- 'active' | 'completed' | 'cancelled' | 'expired'
    state                       JSONB NOT NULL DEFAULT '{}', -- agent's own scratch data (e.g. partial booking fields)
    started_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_activity_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at                    TIMESTAMPTZ
);

-- at most one active sub-agent per (session, speaker) at a time, enforced at the DB level.
-- Scoped per-speaker, not just per-session, because a session can be a group chat with
-- multiple independent participants — see 2026-07-22-cross-channel-identity-design.md.
-- In a DM, sender_channel_identity_id is always the session's one participant, so this
-- is equivalent to "one active agent per session" there.
CREATE UNIQUE INDEX agent_sessions_one_active_per_speaker
    ON agent_sessions (session_id, sender_channel_identity_id) WHERE status = 'active';
```

## State machine

Four states:

```
              spawn                                complete_task(status=completed)
   (none) ─────────────► active ───────────────────────────────────────────────► completed
                            │                                                    (thread hands back
                            │ complete_task(status=cancelled)                     to chitchat)
                            ├───────────────────────────────────────────────► cancelled
                            │  (agent itself recognizes "stop"/"nevermind"
                            │   and calls the same completion tool)
                            │
                            │ next inbound message AND
                            │ now() - last_activity_at > timeout
                            └───────────────────────────────────────────────► expired
```

- **Spawn**: router sees no `active` row for `(session_id, sender_channel_identity_id)` → classifies intent → if money/booking, inserts a new `agent_sessions` row (`status='active'`), publishes `AgentSpawned` to MQTT + `agent_events`.
- **Continue**: router sees an `active` row for `(session_id, sender_channel_identity_id)` → routes the message straight to that agent (no re-classification), passing its `state` JSONB back in. Scoping by sender as well as session means, in a group chat, each participant's active-agent state is independent (see `2026-07-22-cross-channel-identity-design.md`); in a DM it behaves exactly as "per session," since there's only one participant.
- **Complete/Cancel**: every sub-agent's tool registry includes a shared `complete_task(status: completed|cancelled, summary: text)` tool. Calling it updates the row (`status`, `ended_at`), publishes `AgentCompleted`/`AgentCancelled`. The agent itself is responsible for recognizing cancellation phrasing ("stop," "nevermind") and calling this tool with `status=cancelled` — there is no separate orchestrator-level cancel-phrase check, to keep the orchestrator routing logic simple and keep this behavior consistent with how completion already works.

  **Accepted risk**: this makes cancellation correctness a per-sub-agent responsibility rather than a platform guarantee. Every future sub-agent's system prompt must reliably recognize cancel intent, or a user can get stuck unable to exit that agent except via the idle-timeout expiry. Mitigation: this must be part of the standard sub-agent template/checklist used when building any new sub-agent (a required test case: "does typing 'stop' exit this agent"), not left to each implementer's discretion.
- **Expire**: checked lazily, not via a background sweep. When a message arrives for a session with an `active` row whose `last_activity_at` is older than a timeout (default 24h, tunable), the orchestrator marks it `expired`, publishes `AgentExpired`, and routes the incoming message fresh through chitchat instead of the stale agent.

Every transition updates `last_activity_at` and is published as an event, consistent with the parent plan's "everything is broadcast + audited" principle.

## Concurrency: serialization via advisory lock

Per-session serialization is enforced with a Postgres **session-scoped** advisory lock (not transaction-scoped — see Transaction Boundary below for why):

```sql
SELECT pg_advisory_lock(hashtext(session_id::text));
-- ... process the turn (Txn A, then Txn B) ...
SELECT pg_advisory_unlock(hashtext(session_id::text));
```

- Keyed on a hash of `session_id` — two different sessions never contend.
- Lives in Postgres rather than process memory, so this holds correctly even with multiple orchestrator instances — doesn't box the system into a single-instance deployment.
- A second inbound message for the same session simply blocks on this call until the first turn releases the lock, then proceeds. Postgres acts as the queue; no separate queue data structure needed.
- The lock **must** be released in a `finally`/on-error path, not only on the happy path, or one failed turn wedges every subsequent message for that session.
- **Connection pinning is required.** `pg_advisory_lock` is scoped to the physical connection/session that acquired it, not to a transaction. Since Txn A and Txn B are separate transactions, the implementation must hold a single dedicated connection (checked out once from the pool) for the entire acquire → Txn A → Txn B → release span. If a pool hands Txn A and Txn B different connections, the lock provides no real serialization and `pg_advisory_unlock` may run against the wrong session.

## Transaction boundary (correctness fix)

The parent plan promises: *"message arrives → appended to `messages` immediately, so the thread never loses a turn even if something downstream fails."* A single transaction spanning message-append through reply-persist would violate this — if the downstream agent/LLM call fails, rolling back would also erase the inbound message.

Fix: split into two transactions inside one held advisory lock:

1. **Txn A** (fast, always succeeds): append inbound message to `messages`, commit immediately. Message is now durable regardless of what happens next.
2. **Txn B**: look up/route `agent_sessions`, run the turn (chitchat or sub-agent, may call `complete_task`), persist reply, update `agent_sessions` state, publish events, commit.

Full turn loop:

1. Acquire session-scoped advisory lock on `session_id`.
2. **Txn A**: append inbound message, commit.
3. Look up `agent_sessions` row for `(session_id, sender_channel_identity_id)` where `status='active'`.
   - Found and not stale → route to that agent with its `state`.
   - Found and stale (`last_activity_at` > timeout) → mark `expired`, fall through to chitchat routing.
   - Not found → classify intent: chitchat, or spawn new `agent_sessions` row.
4. **Txn B**: run the turn, persist reply, update `agent_sessions`, publish events, commit.
5. Release advisory lock (always — success or failure).

## Error handling

If Txn B fails (LLM error, tool exception, timeout):

- The user's message is already durable (Txn A committed) — nothing is lost.
- No reply is persisted; no `agent_sessions` state changes, so an active sub-agent remains in its prior state rather than being corrupted mid-update.
- An `AgentTurnFailed` event is published to `agent_events` best-effort, outside Txn B, so the failure is still visible in the audit trail.
- The advisory lock releases regardless of outcome, so the next message for this session is not blocked by one failed turn.
- No automatic retry. The user's next message (or a resend) naturally re-enters at step 1 — simpler than a retry queue, and consistent with "it's just a chat": a missing reply reads as the assistant not having answered yet.

## Testing approach

- **Unit**: state transition logic (spawn/continue/complete/cancel/expire) against a fake `agent_sessions` row, no DB required.
- **Integration** (real Postgres): the partial unique index rejects a second `active` row for the same `session_id`; the advisory lock serializes two concurrent turns for the same session (second blocks until first releases); Txn A's commit survives when Txn B is forced to fail.
- **Crash simulation**: kill the process mid-Txn-B and assert the advisory lock isn't left held and the Txn-A message is still present on restart.

## Out of scope (deferred to later designs)

- Cross-channel/cross-session user identity resolution (next design).
- Reinforcement/memory attribution schema.
- Proactive re-engagement (e.g. nudging a user whose sub-agent is about to expire) — the lazy-expiry approach chosen here doesn't preclude adding this later via a background sweep, but it isn't built now.
