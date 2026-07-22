# Cross-Channel Identity & Group Chat Design

Date: 2026-07-22
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md`
Related: `2026-07-22-sub-agent-lifecycle-design.md` (this doc amends its `agent_sessions` schema)

## Purpose

The parent plan's §5 open decision asks: "if the same person messages via Telegram and WhatsApp, do they map to one `user_id` or stay separate threads?" This document resolves that, plus a related gap raised during discussion: channels like Telegram/WhatsApp groups contain *multiple* users in one conversation, which the original single-`channel_identity`-per-session model can't represent.

Scope: cross-channel linking is designed for later (v1 ships on one channel), but group chat support is needed at MVP, so both are designed now rather than only linking.

## Schema: person vs. channel handle

```sql
CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE channel_identities (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users(id),
    channel           TEXT NOT NULL,        -- 'telegram' | 'whatsapp' | future
    channel_user_id   TEXT NOT NULL,        -- e.g. Telegram user id, WhatsApp phone number
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (channel, channel_user_id)
);
```

`users.id` is the canonical person — `memory_items`, `agent_events`, and reinforcement `weight` are all keyed on it. `channel_identities` is the per-channel handle, resolved to a `user_id` for personalization purposes wherever needed.

## Bootstrap (lazy identity creation)

No signup step. Two independent lookups happen on every inbound message, since "is this a new person" and "is this a new chat" are different questions (a group chat can already exist with an established history while still containing a person the bot has never seen before):

1. **Identity**: look up `channel_identities` by `(channel, channel_user_id-of-sender)`. If not found, create a `users` row, then a `channel_identities` row pointing at it.
2. **Session**: look up `sessions` by `(channel, chat_id)` (chat_id is the DM partner's id or the group's id — see Sessions & Groups below). If not found, create it (`chat_type='dm'` or `'group'` per the inbound event's shape).

A brand-new DM creates both a new user and a new session in the same turn. A message from a never-seen participant in an already-active group creates only a new user/identity — the session already exists. Either way, every user starts as a "channel of one"; linking is opt-in later, not a gate before first use.

## Cross-channel linking (designed now, not built for v1)

1. From an existing channel (A), user requests a link (e.g. "link my WhatsApp" in chitchat, or a `/link` command).
2. Orchestrator generates a short-lived, single-use code:

```sql
CREATE TABLE link_codes (
    code        TEXT PRIMARY KEY,    -- 6 chars, unambiguous charset (no 0/O/1/I)
    user_id     UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL,  -- created_at + 10 minutes
    used_at     TIMESTAMPTZ
);
```

3. User sends that code as a message on channel B.
4. Orchestrator recognizes a bare code-shaped message as a link attempt: looks it up, and if found/unexpired/unused, merges B's `channel_identities.user_id` to point at A's canonical `user_id`, and reassigns any of B's pre-existing `memory_items`/`agent_events` rows (tagged under B's old, now-discarded `user_id`) to the canonical one. Marks the code `used_at` so it can't be replayed.
5. Invalid/expired code → plain rejection reply. Rate-limit code-attempt guesses per channel identity (e.g. 5 tries then cooldown) to prevent brute-forcing the 6-char space.

**Design choice**: linking merges history (channel B's existing memory isn't orphaned), and threads stay separate per channel even after linking — Telegram and WhatsApp keep their own conversation histories, but both resolve to the same `user_id` for memory/personalization/reinforcement. No cross-channel session merging, so no cross-channel advisory lock is needed.

## Sessions & groups

A session is keyed by chat, not by a single identity — a DM and a group are both just "a chat," the difference is how many `channel_identities` participate in it.

```sql
ALTER TABLE sessions
    ADD COLUMN chat_type TEXT NOT NULL DEFAULT 'dm',  -- 'dm' | 'group'
    ADD COLUMN chat_id   TEXT NOT NULL;                -- DM: the person's channel_user_id; group: the platform's group id
ALTER TABLE sessions ADD CONSTRAINT sessions_channel_chat_unique UNIQUE (channel, chat_id);

ALTER TABLE messages
    ADD COLUMN sender_channel_identity_id UUID REFERENCES channel_identities(id); -- NULL = assistant
```

**Context model in groups**: per-speaker within a shared thread. Everyone in the group sees one message history (matches the real chat), but every message is tagged with its sender, and any memory retrieval / RAG / personalization for a reply is scoped to whichever `channel_identity` (→ `user_id`) just spoke — not to the group as a collective. A money-agent reply to Alice pulls Alice's `memory_items`, never Bob's.

**Reply privacy**: the bot only ever replies where the message came from — no auto-DM-switching for sensitive sub-agents. If Alice engages the money agent inside a group, replies are visible to the group; that's an accepted, obvious tradeoff of using a sub-agent inside a shared chat, not something the platform tries to route around in v1.

## Amendment to sub-agent lifecycle design: per-speaker agent scoping

Groups break the lifecycle spec's original invariant of "one active sub-agent per `session_id`" — a session now has many possible speakers, each of whom may independently be mid-task. `agent_sessions` is amended:

```sql
ALTER TABLE agent_sessions
    ADD COLUMN sender_channel_identity_id UUID NOT NULL REFERENCES channel_identities(id);

DROP INDEX agent_sessions_one_active;
CREATE UNIQUE INDEX agent_sessions_one_active_per_speaker
    ON agent_sessions (session_id, sender_channel_identity_id) WHERE status = 'active';
```

In a DM, `sender_channel_identity_id` is always the one person in that session, so behavior there is unchanged from the original spec. In a group, Alice's and Bob's active-agent states are fully independent even though they share one thread.

## Turn loop (final version, supersedes the lifecycle spec's loop)

1. Acquire session-scoped advisory lock on `session_id` (unchanged — the whole chat's turns still serialize one at a time; in a group this also prevents two bot replies from interleaving in front of everyone).
2. Resolve `sender_channel_identity_id` from the inbound payload (in a group, the platform gives a distinct sender id separate from the group's `chat_id`); bootstrap a new `channel_identities`/`users` row if this sender has never been seen on any channel.
3. **Txn A**: append message tagged with `sender_channel_identity_id`, commit. (Durable regardless of what happens downstream — see lifecycle spec's Transaction Boundary section, unchanged.)
4. Look up `agent_sessions` for `(session_id, sender_channel_identity_id)` — same active/stale/spawn/complete/cancel state machine as the lifecycle spec, now scoped to this specific speaker.
5. Resolve the sender's `user_id` for memory/RAG lookup — personalization always resolves to the *speaker's* `user_id`, regardless of which shared session (DM or group) they spoke in.
6. **Txn B**: run the turn, persist reply, update `agent_sessions`, publish events tagged with `sender_channel_identity_id`, commit.
7. Release advisory lock (always).

## Out of scope

- Actually building cross-channel linking UI/commands for v1 (schema is ready; feature ships later per plan §5 scoping).
- Auto-DM-switching for sensitive sub-agent replies in groups (explicitly rejected above for v1).
- Group-level settings (e.g. muting the bot, per-group sub-agent permissions) — not raised yet, worth its own design if needed later.
