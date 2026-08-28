# Admin Dashboard and Agent List Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the admin section's placeholder dashboard into a real one (total users, token usage today/all-time, running agents), and add a new page listing every currently-active agent session grouped by the user it belongs to.

**Architecture:** Two new read-only HTTP routes in `nomi-server` (`GET /api/admin/dashboard`, `GET /api/admin/agents`), both gated by the existing `require_system_config_permission` check every other admin route already uses, querying only tables that already exist (`users`, `agent_sessions`, `agent_events`, `channel_identities`, `web_credentials`) — no migration needed. The frontend replaces the placeholder `/admin` page with real stat cards and adds a new `/admin/agents` page, both following the LLM-models admin page's existing `apiFetch`-backed `load` pattern and MD3 `Card` component.

**Tech Stack:** Rust, axum, sqlx (Postgres), SvelteKit 2 / Svelte 5, Tailwind, MD3 design tokens.

**Spec:** `docs/superpowers/specs/2026-08-28-admin-dashboard-and-agents-design.md`

## Global Constraints

- Both new routes are system-wide, not scoped to the calling admin's own org — matching how the existing LLM-models admin page manages system-wide models (spec §1).
- Both new routes reuse `require_system_config_permission` (`backend/crates/nomi-server/src/routes/settings.rs`) — no new permission model, no new route-level auth logic.
- No new migration: this feature is read-only against tables that already exist.
- **Postgres gotcha that affects the token-sum query:** `SUM(bigint_expression)` returns `NUMERIC` in Postgres, not `BIGINT` — sqlx's default `i64` mapping only works against a `BIGINT` column. Every aggregate query in this plan that sums the `bigint`-cast token fields must wrap the whole `COALESCE(...)` in an explicit `::bigint` cast so sqlx decodes it as `i64`, not a `NUMERIC`/decimal type this workspace has no feature flag for.
- The agents list groups by `user_id`, not by `channel_identity_id` — a person active on both Telegram and the web at once must appear as one group (spec §1).
- No automated frontend tests are added — the codebase has none for any admin or chat page shipped so far. Verification for both frontend tasks is `npm run check` plus self-review, not a test run.
- Only use MD3 typescale classes that already exist in `frontend/src/lib/styles/material3.css` (confirmed set: `md-headline-small`, `md-title-large`, `md-title-medium`, `md-body-large`, `md-body-medium`, `md-body-small`, `md-label-large`, `md-label-medium` — no `md-headline-medium` or similar exists, don't invent one).

---

### Task 1: Backend — `GET /api/admin/dashboard` and `GET /api/admin/agents`

**Files:**
- Create: `backend/crates/nomi-server/src/routes/admin_dashboard.rs`
- Modify: `backend/crates/nomi-server/src/routes/mod.rs`
- Modify: `backend/crates/nomi-server/src/app.rs`
- Create: `backend/crates/nomi-server/tests/admin_dashboard_routes.rs`

**Interfaces:**
- Produces (used by Task 2): `GET /api/admin/dashboard` → `{"total_users": i64, "tokens_today": i64, "tokens_all_time": i64, "running_agents": i64}`.
- Produces (used by Task 3): `GET /api/admin/agents` → `{"users": [{"user_id": Uuid, "label": String, "agents": [{"agent_session_id": Uuid, "agent_type": String, "channel": String, "started_at": DateTime<Utc>, "last_activity_at": DateTime<Utc>}]}]}`.

- [ ] **Step 1: Implement the routes**

Create `backend/crates/nomi-server/src/routes/admin_dashboard.rs`:

```rust
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::routes::settings::require_system_config_permission;
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct DashboardResponse {
    pub total_users: i64,
    pub tokens_today: i64,
    pub tokens_all_time: i64,
    pub running_agents: i64,
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<DashboardResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to count users"))?;

    let running_agents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_sessions WHERE status = 'active'")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to count running agents"))?;

    let (tokens_all_time, tokens_today): (i64, i64) = sqlx::query_as(
        "SELECT \
             COALESCE(SUM((payload->>'input_tokens')::bigint + (payload->>'output_tokens')::bigint), 0)::bigint AS all_time, \
             COALESCE(SUM((payload->>'input_tokens')::bigint + (payload->>'output_tokens')::bigint) \
                       FILTER (WHERE created_at >= date_trunc('day', now())), 0)::bigint AS today \
         FROM agent_events WHERE event_type = 'AgentReplied'",
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to sum token usage"))?;

    Ok(Json(DashboardResponse { total_users, tokens_today, tokens_all_time, running_agents }))
}

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

pub async fn get_agents(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<AgentsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;

    let rows: Vec<(Uuid, Option<String>, String, String, Uuid, String, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT \
             u.id, wc.email, ci.channel, ci.channel_user_id, \
             ags.id, ags.agent_type, ags.started_at, ags.last_activity_at \
         FROM agent_sessions ags \
         JOIN channel_identities ci ON ci.id = ags.sender_channel_identity_id \
         JOIN users u ON u.id = ci.user_id \
         LEFT JOIN web_credentials wc ON wc.user_id = u.id \
         WHERE ags.status = 'active' \
         ORDER BY u.id, ags.started_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load running agents"))?;

    // Rows are ORDER BY u.id, so every row for the same user is contiguous — grouping by
    // checking the last-pushed group's user_id needs no HashMap or second pass.
    let mut groups: Vec<UserAgentGroup> = Vec::new();
    for (user_id, email, channel, channel_user_id, agent_session_id, agent_type, started_at, last_activity_at) in rows {
        let agent = RunningAgentItem { agent_session_id, agent_type, channel: channel.clone(), started_at, last_activity_at };
        match groups.last_mut() {
            Some(group) if group.user_id == user_id => group.agents.push(agent),
            _ => {
                let label = email.unwrap_or_else(|| format!("{channel}:{channel_user_id}"));
                groups.push(UserAgentGroup { user_id, label, agents: vec![agent] });
            }
        }
    }

    Ok(Json(AgentsResponse { users: groups }))
}
```

- [ ] Edit `backend/crates/nomi-server/src/routes/mod.rs` — add the module:

```rust
pub mod admin_dashboard;
pub mod auth;
pub mod llm_models;
pub mod personality;
pub mod sessions;
pub mod settings;
```

- [ ] Edit `backend/crates/nomi-server/src/app.rs` — add the import and two routes. Add this import alongside the other route-module imports:

```rust
use crate::routes::admin_dashboard as admin_dashboard_routes;
```

Add these two routes to the chain (placed here after the `personality` routes for locality, order doesn't matter):

```rust
        .route("/api/admin/dashboard", get(admin_dashboard_routes::get_dashboard))
        .route("/api/admin/agents", get(admin_dashboard_routes::get_agents))
```

- [ ] **Step 2: Verify the workspace builds**

```bash
cd backend && cargo build --workspace
```

Expected: builds cleanly.

- [ ] **Step 3: Write the tests**

Create `backend/crates/nomi-server/tests/admin_dashboard_routes.rs`:

```rust
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
    }
}

async fn json_request(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn register_via_api(router: axum::Router, email: &str) {
    json_request(
        router,
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
}

async fn login_via_api(router: axum::Router, email: &str) -> String {
    let (_, login_body) = json_request(
        router,
        "POST",
        "/api/auth/login",
        json!({ "email": email, "password": "correct-password" }),
        None,
    )
    .await;
    login_body["access_token"].as_str().unwrap().to_string()
}

async fn make_platform_admin(pool: &PgPool, email: &str) {
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    register_via_api(router.clone(), email).await;
    make_platform_admin(pool, email).await;
    login_via_api(router, email).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden_from_viewing_the_dashboard(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/dashboard", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden_from_listing_agents(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/agents", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn dashboard_returns_zero_counts_on_a_fresh_install(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/admin/dashboard", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    // The admin's own registration is the only user row, and nothing else has happened yet —
    // COALESCE(SUM(...), 0) must report 0, not null or an error, for the token fields.
    assert_eq!(body["total_users"], 1);
    assert_eq!(body["running_agents"], 0);
    assert_eq!(body["tokens_today"], 0);
    assert_eq!(body["tokens_all_time"], 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn dashboard_reports_correct_counts(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "regular@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'tg-1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // One active session, one completed — running_agents must count only the active one.
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'chitchat', 'active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'completed')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    // One AgentReplied event today, one backdated two days — proves the today/all-time split.
    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_type, event_type, payload, created_at) \
         VALUES ($1, 'chitchat', 'AgentReplied', $2, now())",
    )
    .bind(session_id)
    .bind(json!({"input_tokens": 100, "output_tokens": 50}))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_type, event_type, payload, created_at) \
         VALUES ($1, 'chitchat', 'AgentReplied', $2, now() - interval '2 days')",
    )
    .bind(session_id)
    .bind(json!({"input_tokens": 200, "output_tokens": 75}))
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = json_request(router, "GET", "/api/admin/dashboard", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_users"], 2);
    assert_eq!(body["running_agents"], 1);
    assert_eq!(body["tokens_today"], 150);
    assert_eq!(body["tokens_all_time"], 425);
}

#[sqlx::test(migrations = "../../migrations")]
async fn agents_endpoint_groups_running_sessions_by_user_and_excludes_non_active(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // User A: has a web account, one active agent session.
    let user_a: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO web_credentials (user_id, email, password_hash) VALUES ($1, 'a@example.com', 'x')")
        .bind(user_a)
        .execute(&pool)
        .await
        .unwrap();
    let identity_a: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'a-tg') RETURNING id",
    )
    .bind(user_a)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'chitchat', 'active')",
    )
    .bind(session_id)
    .bind(identity_a)
    .execute(&pool)
    .await
    .unwrap();

    // User B: no web account (channel-only), one active session and one completed (excluded).
    let user_b: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let identity_b: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'b-tg') RETURNING id",
    )
    .bind(user_b)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active')",
    )
    .bind(session_id)
    .bind(identity_b)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'personality', 'completed')",
    )
    .bind(session_id)
    .bind(identity_b)
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = json_request(router, "GET", "/api/admin/agents", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    let users = body["users"].as_array().unwrap();
    assert_eq!(users.len(), 2);

    let group_a = users.iter().find(|g| g["user_id"] == user_a.to_string()).unwrap();
    assert_eq!(group_a["label"], "a@example.com");
    assert_eq!(group_a["agents"].as_array().unwrap().len(), 1);

    let group_b = users.iter().find(|g| g["user_id"] == user_b.to_string()).unwrap();
    assert_eq!(group_b["label"], "telegram:b-tg");
    assert_eq!(group_b["agents"].as_array().unwrap().len(), 1);
    assert_eq!(group_b["agents"][0]["agent_type"], "money");
}
```

- [ ] **Step 4: Run the tests**

```bash
cd backend && cargo test -p nomi-server --test admin_dashboard_routes
```

Expected: all 5 tests PASS.

- [ ] **Step 5: Run the whole `nomi-server` test suite**

```bash
cd backend && cargo test -p nomi-server
```

Expected: all tests PASS.

- [ ] **Step 6: Commit**

```bash
git add backend/crates/nomi-server/src/routes/admin_dashboard.rs backend/crates/nomi-server/src/routes/mod.rs \
        backend/crates/nomi-server/src/app.rs backend/crates/nomi-server/tests/admin_dashboard_routes.rs
git commit -m "feat: add admin dashboard and running-agents HTTP routes"
```

---

### Task 2: Frontend — the real `/admin` dashboard page

**Files:**
- Modify: `frontend/src/lib/types.ts`
- Create: `frontend/src/routes/admin/(protected)/+page.server.ts`
- Modify: `frontend/src/routes/admin/(protected)/+page.svelte`

**Interfaces:**
- Consumes: `GET /api/admin/dashboard` (Task 1).

- [ ] **Step 1: Add the type**

Edit `frontend/src/lib/types.ts` — add at the end of the file:

```ts
export interface DashboardStats {
	total_users: number;
	tokens_today: number;
	tokens_all_time: number;
	running_agents: number;
}
```

- [ ] **Step 2: Add the load function**

Create `frontend/src/routes/admin/(protected)/+page.server.ts` (this page currently has no server file — only `+page.svelte`):

```ts
import { apiFetch } from '$lib/server/api';
import type { DashboardStats } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/dashboard');
	const stats: DashboardStats = response.ok
		? ((await response.json()) as DashboardStats)
		: { total_users: 0, tokens_today: 0, tokens_all_time: 0, running_agents: 0 };
	return { stats };
};
```

- [ ] **Step 3: Replace the placeholder page**

Replace the full contents of `frontend/src/routes/admin/(protected)/+page.svelte`:

```svelte
<script lang="ts">
	import Card from '$lib/components/m3/Card.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const cards = $derived([
		{ label: 'Total Users', value: data.stats.total_users },
		{ label: 'Tokens Today', value: data.stats.tokens_today },
		{ label: 'Tokens All-Time', value: data.stats.tokens_all_time },
		{ label: 'Running Agents', value: data.stats.running_agents },
	]);
</script>

<h1 class="md-headline-small" style="color: var(--md-sys-color-on-surface)">Admin dashboard</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Manage app-wide configuration.
</p>

<div class="mt-6 grid grid-cols-2 gap-4 md:grid-cols-4">
	{#each cards as card (card.label)}
		<Card variant="outlined" class="p-4">
			<p class="md-label-medium" style="color: var(--md-sys-color-on-surface-variant)">{card.label}</p>
			<p class="md-title-large mt-1" style="color: var(--md-sys-color-on-surface)">{card.value.toLocaleString()}</p>
		</Card>
	{/each}
</div>
```

- [ ] **Step 4: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors.

- [ ] **Step 5: Self-review**

Confirm: the four cards render in the order Total Users, Tokens Today, Tokens All-Time, Running Agents; numbers are formatted with `toLocaleString()` (so e.g. `12345` reads `12,345`); the page still starts with the same headline/subhead text as before, just with the stat grid added beneath.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/lib/types.ts frontend/src/routes/admin/\(protected\)/+page.server.ts \
        frontend/src/routes/admin/\(protected\)/+page.svelte
git commit -m "feat: show real dashboard stats on the admin home page"
```

---

### Task 3: Frontend — new `/admin/agents` page, nav link, and icon

**Files:**
- Modify: `frontend/src/lib/types.ts`
- Modify: `frontend/src/lib/components/m3/Icon.svelte`
- Modify: `frontend/src/routes/admin/(protected)/+layout.svelte`
- Create: `frontend/src/routes/admin/(protected)/agents/+page.server.ts`
- Create: `frontend/src/routes/admin/(protected)/agents/+page.svelte`

**Interfaces:**
- Consumes: `GET /api/admin/agents` (Task 1).

- [ ] **Step 1: Add the types**

Edit `frontend/src/lib/types.ts` — add at the end of the file:

```ts
export interface RunningAgentItem {
	agent_session_id: string;
	agent_type: string;
	channel: string;
	started_at: string;
	last_activity_at: string;
}

export interface UserAgentGroup {
	user_id: string;
	label: string;
	agents: RunningAgentItem[];
}

export interface AgentsResponse {
	users: UserAgentGroup[];
}
```

- [ ] **Step 2: Add the `agents` icon**

Edit `frontend/src/lib/components/m3/Icon.svelte` — add `'agents'` to the `IconName` union:

```ts
	export type IconName =
		| 'dashboard'
		| 'settings'
		| 'logout'
		| 'plus'
		| 'chat-bubble'
		| 'chevron-left'
		| 'chevron-right'
		| 'agents';
```

Add a new branch to the SVG's `{#if}` chain, right before the closing `{/if}`:

```svelte
	{:else if name === 'agents'}
		<rect x="5" y="8" width="14" height="10" rx="2" />
		<line x1="12" y1="8" x2="12" y2="4" />
		<circle cx="12" cy="3" r="1" />
		<circle cx="9" cy="13" r="1" />
		<circle cx="15" cy="13" r="1" />
```

- [ ] **Step 3: Add the nav link**

Edit `frontend/src/routes/admin/(protected)/+layout.svelte` — in the collapsed nav block, add after the existing "LLM Settings" icon link:

```svelte
				<a href="/admin/agents" class="m3-icon-button" aria-label="Agents">
					<Icon name="agents" />
				</a>
```

In the expanded nav block, add after the existing "LLM Settings" text link:

```svelte
				<a href="/admin/agents" class="m3-nav-link">Agents</a>
```

- [ ] **Step 4: Add the load function**

Create `frontend/src/routes/admin/(protected)/agents/+page.server.ts`:

```ts
import { apiFetch } from '$lib/server/api';
import type { AgentsResponse } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/agents');
	const agents: AgentsResponse = response.ok ? ((await response.json()) as AgentsResponse) : { users: [] };
	return { agents };
};
```

- [ ] **Step 5: Add the page**

Create `frontend/src/routes/admin/(protected)/agents/+page.svelte`:

```svelte
<script lang="ts">
	import Card from '$lib/components/m3/Card.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<h1 class="md-headline-small" style="color: var(--md-sys-color-on-surface)">Running agents</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Every currently active agent session, grouped by user.
</p>

<div class="mt-6 space-y-4">
	{#if data.agents.users.length === 0}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
			No agents are currently running.
		</p>
	{:else}
		{#each data.agents.users as group (group.user_id)}
			<Card variant="outlined" class="p-4">
				<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{group.label}</p>
				<div class="mt-2 space-y-1">
					{#each group.agents as agent (agent.agent_session_id)}
						<div class="md-body-medium flex justify-between" style="color: var(--md-sys-color-on-surface-variant)">
							<span>{agent.agent_type} · {agent.channel}</span>
							<span>started {new Date(agent.started_at).toLocaleString()}</span>
						</div>
					{/each}
				</div>
			</Card>
		{/each}
	{/if}
</div>
```

- [ ] **Step 6: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors.

- [ ] **Step 7: Self-review**

Confirm: the nav link appears in both the collapsed (icon-only) and expanded sidebar states, in both cases positioned after "LLM Settings"; the new icon renders (no missing-glyph gap) at both 20px (nav) and any other size the icon component is used at elsewhere; the empty state message shows when no agents are running; each user group's agents are listed with type, channel, and a formatted start time.

- [ ] **Step 8: Commit**

```bash
git add frontend/src/lib/types.ts frontend/src/lib/components/m3/Icon.svelte \
        frontend/src/routes/admin/\(protected\)/+layout.svelte \
        frontend/src/routes/admin/\(protected\)/agents/+page.server.ts \
        frontend/src/routes/admin/\(protected\)/agents/+page.svelte
git commit -m "feat: add the admin running-agents page with nav link"
```
