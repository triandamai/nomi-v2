That's the consolidated picture — one Postgres+pgvector instance is now the entire "brain" (sessions, RAG memory, audit log), MQTT is purely the live event bus layered on top of it. Here's the full plan tied together.

## 1. What the system is
A Rust-native orchestrator that holds one persistent, chat-app-style conversation thread per user per channel. Most turns are answered directly (chitchat, RAG-augmented). Some turns get handed off to a short-lived sub-agent (money, booking) that has its own scoped tools, then hands the thread back — the user experiences it as one continuous chat, never a "new session."

## 2. The layers

**Channels** — Telegram (`teloxide`, easy), WhatsApp (Cloud API over HTTP, or a bridged unofficial lib since there's no native Rust SDK), extensible to Slack/web/Discord. Each adapter just normalizes inbound messages into `(channel, user_id, text)` and forwards outbound replies.

**Orchestrator core** — owns the turn loop: load session → RAG retrieval → route → answer or delegate → persist → publish event. This is the only thing that talks to every other layer; agents and channels don't talk to each other directly.

**Multi-model layer** — a provider trait so any agent/turn can pick the model that fits (cheap/local for chitchat and intent classification, a stronger hosted model for financial reasoning or multi-step planning). This is what "multi-model" buys you: cost and capability control per turn, not per app.

**Agent spawner + sub-agents** — chitchat is the default, always-on agent. Money and booking are spawned on demand, each with its own system prompt and a *hard* tool allow-list — the money agent, being advisory-only for now, literally has no transfer/payment tool wired into its registry, so there's no prompt-injection path to a real transaction even if the model tries.

**MQTT event bus** — every spawn, tool call, completion, and feedback signal is broadcast here live. This is what a UI, a notification service, or a separate reinforcement worker would subscribe to, without being coupled to the orchestrator's internals.

**Postgres + pgvector (the brain)** — three tables, one database:
- `sessions` / `messages`: the permanent chat thread, exactly like a WhatsApp conversation — never expires, just accumulates.
- `memory_items`: long-term memory with a `VECTOR` column, retrieved via cosine similarity for RAG.
- `agent_events`: durable audit trail, the twin of what MQTT broadcasts live — important for the money/booking agents where you'll want a real paper trail.

## 3. How a turn actually flows
1. Message arrives → appended to `messages` immediately (so the thread never loses a turn even if something downstream fails).
2. Text gets embedded → pgvector similarity search over `memory_items`, filtered by user, re-ranked by `weight` (the reinforcement signal) alongside raw similarity.
3. Router decides: continue an active sub-agent, spawn a new one, or answer as chitchat with retrieved memories + recent history folded into the prompt.
4. Reply persisted, event published.
5. Later, feedback (thumbs up/down, task outcome, explicit correction) triggers `reinforce()`, which nudges the `weight` of whichever memory contributed to that answer — up if it helped, down if it led to a correction. No model retraining, no separate RL pipeline — just retrieval that gets better calibrated to this specific user over time.

## 4. Money agent guardrails (advisory-only scope)
- Read/categorize only — the tool registry for this agent contains things like "list transactions," "categorize spending," "summarize budget," nothing that moves funds.
- Every recommendation it gives is logged to `agent_events` so there's an audit trail even though nothing executes.
- This is the natural on-ramp to read-only bank/PSP API integration later, and to real transactions only after you're comfortable with a human-approval-event step (`ActionPendingApproval` → `ActionApproved` already modeled in the event schema for exactly this).

## 5. Open decisions before implementation starts
- **Embedding model + dimension** — this fixes the `VECTOR(n)` column size in the migration, so it's worth locking in early (e.g. 1536 for OpenAI's text-embedding-3-small, different for local models).
- **Postgres hosting** — self-hosted vs. managed (Supabase, Neon, RDS) — matters for how you enable the `vector` extension.
- **MQTT broker** — Mosquitto (simple, self-hosted) vs. EMQX (more features, clustering) vs. a managed option.
- **First channel + first reservation vertical** — narrowing scope for the MVP (e.g. Telegram + restaurant bookings only) before widening.
- **Cross-channel identity** — if the same person messages via Telegram and WhatsApp, do they map to one `user_id` or stay separate threads? Affects the `sessions` unique key.
- **Model providers for v1** — which hosted APIs plus whether a local model is in scope from day one or added later.

Let me know when you want to lock any of these in, or move to a phased implementation plan against this architecture.