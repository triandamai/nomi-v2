# Admin Dashboard and Agent List

Date: 2026-08-28
Status: Approved (pending user review of this doc)
Related: none — this is the first admin-visibility feature beyond LLM model settings, but reuses that page's exact patterns (`require_system_config_permission`, the M3 nav-rail sidebar, `apiFetch`-backed SvelteKit `load` functions).

## Purpose

The admin section (`/admin`, gated behind the `nomi:admin:system_config:manage` permission) currently has one real page — LLM model settings — plus a placeholder dashboard that just says "Admin dashboard" with no content. This design fills that placeholder in with four at-a-glance numbers (total users, token usage today and all-time, currently running agents), and adds a second new page listing every currently-running agent session, grouped by the user it belongs to.

## 1. Backend — Two New Admin Routes

Both routes live in a new `backend/crates/nomi-server/src/routes/admin_dashboard.rs`, gated by the same `require_system_config_permission(&claims)` check `routes/settings.rs` and `routes/llm_models.rs` already use — no new permission model. Both are system-wide (not scoped to the calling admin's own org), matching how the LLM-models page already manages system-wide models rather than per-org ones.

### `GET /api/admin/dashboard`

```rust
#[derive(Serialize)]
pub struct DashboardResponse {
    pub total_users: i64,
    pub tokens_today: i64,
    pub tokens_all_time: i64,
    pub running_agents: i64,
}
```

Three queries:

```sql
-- total_users
SELECT COUNT(*) FROM users

-- running_agents
SELECT COUNT(*) FROM agent_sessions WHERE status = 'active'

-- tokens_today / tokens_all_time (one query, one aggregate, filtered twice)
SELECT
    COALESCE(SUM((payload->>'input_tokens')::bigint + (payload->>'output_tokens')::bigint), 0) AS all_time,
    COALESCE(SUM((payload->>'input_tokens')::bigint + (payload->>'output_tokens')::bigint)
              FILTER (WHERE created_at >= date_trunc('day', now())), 0) AS today
FROM agent_events
WHERE event_type = 'AgentReplied'
```

`AgentReplied` events already carry `{input_tokens, output_tokens}` in their JSONB payload for every agent reply, chat or subagent (added when reinforcement/analytics was restored — see `docs/todo/reinforce-memory-usage-gap.md`'s resolution) — no new instrumentation needed. "Today" is a server-timezone day boundary (`date_trunc('day', now())`); no timezone selector, matching the "simplest first cut" scope decision.

### `GET /api/admin/agents`

```rust
#[derive(Serialize)]
pub struct RunningAgentItem {
    pub agent_session_id: Uuid,
    pub agent_type: String,
    pub channel: String,
    pub started_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct UserAgentGroup {
    pub user_id: Uuid,
    pub label: String,
    pub agents: Vec<RunningAgentItem>,
}

#[derive(Serialize)]
pub struct AgentsResponse {
    pub users: Vec<UserAgentGroup>,
}
```

One flat query, grouped into the nested shape in application code (SQL doesn't nest naturally, and this result set is small — every currently-*active* agent session system-wide, not history):

```sql
SELECT
    u.id AS user_id,
    wc.email,
    ci.channel,
    ci.channel_user_id,
    ags.id AS agent_session_id,
    ags.agent_type,
    ags.started_at,
    ags.last_activity_at
FROM agent_sessions ags
JOIN channel_identities ci ON ci.id = ags.sender_channel_identity_id
JOIN users u ON u.id = ci.user_id
LEFT JOIN web_credentials wc ON wc.user_id = u.id
WHERE ags.status = 'active'
ORDER BY u.id, ags.started_at DESC
```

`label` is `wc.email` when the user has a web account, else `"{channel}:{channel_user_id}"` (e.g. `telegram:558212`) for a channel-only user with no web login — every `agent_sessions` row has a real `sender_channel_identity_id`, so this never falls through to an empty label. Grouping happens on `user_id`, not `channel_identity_id`: a person active on both Telegram and the web at once appears as one group with agents from both channels, since `sender_channel_identity_id` only tells you which channel *originated* a given agent session, not which channels that person uses overall.

## 2. Frontend — Dashboard and Agent List Pages

### `/admin` (existing placeholder page becomes the real dashboard)

`+page.server.ts`'s `load` fetches `/api/admin/dashboard`; `+page.svelte` renders four MD3 stat cards (Total Users, Tokens Today, Tokens All-Time, Running Agents) in a grid, replacing the current static headline+paragraph.

### `/admin/agents` (new page)

`+page.server.ts`'s `load` fetches `/api/admin/agents`; `+page.svelte` renders one section per `UserAgentGroup` — a header showing `label`, then a small table/list of that user's `agents` (agent type, channel, started at, last activity). No flat/grouped toggle: since this only ever shows *active* sessions (a small, already-scoped set per the earlier scope decision), grouped-by-user is simply the page's one and only view — there's nothing to toggle away from.

### Nav

`admin/(protected)/+layout.svelte`'s sidebar gains a third link, `/admin/agents` ("Agents"), next to the existing "Dashboard" and "LLM Settings" links, in both the collapsed (icon-only) and expanded nav states. The icon component (`$lib/components/m3/Icon.svelte`) is a closed union of hand-drawn SVG icons with no existing "agents" icon — it gains one new `'agents'` variant (a simple robot/bot glyph, consistent in stroke-width/viewBox with the existing icons).

## 3. Testing

- `nomi-server`: integration tests for both routes (mirroring `tests/personality_routes.rs`'s `test_state`/`json_request`/`register_and_login` helper pattern) — `GET /api/admin/dashboard` returns correct counts against seeded data (users, `agent_sessions`, `agent_events`); `GET /api/admin/agents` correctly groups multiple active sessions under one user and excludes non-active (`completed`/`cancelled`/`expired`) sessions; both routes return 403 for a caller without `system_config:manage`.
- Frontend: no automated tests, consistent with every other admin/chat frontend feature shipped in this codebase so far (zero `.test.ts`/`.spec.ts` files exist anywhere in `frontend/`) — verified via `npm run check`.

## Error Handling

| Condition | Behavior |
|---|---|
| Caller lacks `system_config:manage` | 403, matching every other admin route's existing behavior (`require_system_config_permission`). |
| No agent sessions are currently active | `GET /api/admin/agents` returns `{"users": []}`, not an error — the frontend renders an empty-state message. |
| No `AgentReplied` events exist yet (fresh install) | Token sums are `0`/`0` via `COALESCE`, not null or an error. |

## Out of Scope

- Per-org scoping or filtering of any of these numbers/lists — this section is platform-wide, matching the existing LLM-models admin page.
- Historical/completed agent sessions on the agents page — active only, per the earlier scope decision (a future page could add a history view, but that's a new page, not this one).
- Any kind of time-series chart, cost-in-dollars conversion, or per-provider token breakdown — the dashboard is four plain numbers.
- Killing/cancelling a running agent session from the agents page — this is a read-only visibility feature.
