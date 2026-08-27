# `reinforce()` is unwired for chitchat — memory-reinforcement silently no-ops

**Status:** Fixed (`7734c47`, 2026-08-28)
**Filed:** 2026-08-27
**Introduced by:** `docs/superpowers/plans/2026-08-26-backend-cargo-workspace-implementation.md`, Task 8 (`nomi-agent-chitchat`)
**Severity:** Low today (zero production blast radius, confirmed), but a real correctness gap in a capability that looks fully functional.

## Resolution

Fixed in `7734c47`. `LoopOutcome::Reply` now carries `memory_ids_used: Vec<Uuid>` plus
`input_tokens`/`output_tokens`; `nomi-turn`'s `run_subagent_turn` writes `message_memory_usage`
rows and an `AgentReplied` `agent_events` row from them, inside the same transaction that
persists the reply. `reinforce()` now has real data to act on. Verified end-to-end with
`nomi-turn/tests/turn_handle_inbound_message.rs::a_chitchat_replys_used_memory_is_linked_and_recorded_as_an_agent_replied_event`,
which drives a real chitchat turn through `handle_inbound_message` and asserts the actual
`message_memory_usage` row and `agent_events` row it produces — not just that `LoopOutcome`
compiles with the new fields.

Step 4 from the plan below (wiring an actual thumbs-up/down feature to call `reinforce()`) is
still open and out of scope for this fix — this only restores the plumbing `reinforce()` needs
to have any effect once that feature exists.

---

## Original report (kept for context)

## Summary

`nomi_agent_core::memory::reinforce()` (`backend/crates/nomi-agent-core/src/memory.rs:120`) adjusts a memory's `weight` up or down based on a positive/negative signal, by looking up which memories were used in a given reply:

```rust
"UPDATE memory_items SET weight = LEAST(GREATEST(weight * $1, 0.1), 5.0) \
 WHERE id IN (SELECT memory_id FROM message_memory_usage WHERE message_id = $2)"
```

This depends on `message_memory_usage` rows linking a reply message to the memories that were retrieved and folded into its system prompt. Before the Cargo workspace split, the old bespoke `turn/chitchat.rs` wrote exactly these rows as part of its own reply transaction. That logic did not survive being generalized into `nomi_agent_core::engine::run_agent_turn` (the shared, registry-driven turn engine every agent — including chitchat — now goes through): `LoopOutcome` (`backend/crates/nomi-agent-core/src/engine.rs:32`) only carries `Reply(String)` or `Completed { status, summary }` — no token counts, and no list of which memory IDs were actually retrieved for that turn — so nothing in `nomi-turn`'s `run_subagent_turn` (`backend/crates/nomi-turn/src/lib.rs`) can reconstruct the link after the fact.

**Net effect:** `message_memory_usage` is written by nothing in production. `reinforce()`'s `WHERE id IN (SELECT ...)` matches zero rows for every real reply, forever, as the code stands today.

## Why this wasn't fixed as part of the workspace split

- Restoring it means widening `LoopOutcome`'s shape (e.g. adding a `memory_ids_used: Vec<Uuid>` and/or token counts) — a `nomi-agent-core` engine change, not something the task that discovered the gap (a lightweight "port chitchat into its own crate" task) or the task that rewrote orchestration (`nomi-turn`) naturally owned.
- Confirmed via `grep -rn "reinforce\b"` across the whole workspace: `reinforce()` has **zero production call sites** today — no route, handler, or worker path invokes it. It has its own passing unit tests (`backend/crates/nomi-agent-core/tests/turn_reinforcement.rs`), which is what makes this gap dangerous: the function looks alive and tested, but is fed by a table nothing populates.
- This was independently verified three separate times during the workspace-split plan's review process (the implementer who found it, that task's dedicated reviewer, and the final whole-branch reviewer), all reaching the same conclusion: real gap, zero current impact, correctly out of scope for that plan.

## What needs to happen

1. Decide what `LoopOutcome::Reply` (and possibly `Completed`) should carry to make reinforcement restorable — at minimum the memory IDs that were retrieved for that turn; optionally token counts too, if reintroducing an analytics event (`ChitchatReply`-equivalent) is also wanted.
2. Update `run_agent_turn` (`backend/crates/nomi-agent-core/src/engine.rs`) to populate that data when `agent.uses_memory()` is true.
3. Update `run_subagent_turn` (`backend/crates/nomi-turn/src/lib.rs`) to write `message_memory_usage` rows from it after a `Reply` outcome.
4. Wire up whatever the actual feature is that should call `reinforce()` (e.g. a thumbs-up/down control on a reply) — this ticket only covers restoring the plumbing `reinforce()` needs to have any effect once that feature exists.
5. Add or restore a test proving a real reply → `message_memory_usage` row → `reinforce()` actually adjusts that memory's weight, end to end (not just `reinforce()` in isolation, which already has coverage).

## References

- `backend/crates/nomi-agent-core/src/memory.rs:120` — `reinforce()`
- `backend/crates/nomi-agent-core/src/engine.rs:32` — `LoopOutcome`
- `backend/crates/nomi-agent-core/tests/turn_reinforcement.rs` — existing isolated unit tests (still valid, just untriggered by any real turn)
- `backend/crates/nomi-agent-chitchat/tests/chitchat_agent.rs:119-125` — the test comment documenting exactly what was dropped and why
