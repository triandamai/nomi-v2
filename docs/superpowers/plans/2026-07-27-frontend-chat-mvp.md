# Frontend Chat MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working slice of the SvelteKit frontend described in `docs/superpowers/specs/2026-07-27-frontend-chat-mvp-design.md`: register, log in, see a chat list, start a new chat, and exchange messages with the orchestrator — styled after the "Sand" reference UI — using only the REST endpoints that exist today.

**Architecture:** A new `frontend/` directory at the repo root (sibling to `backend/`), a standalone SvelteKit app (SSR, Tailwind CSS v4, `adapter-node`, `pnpm`). Auth tokens live in httpOnly cookies set by server-side form actions; a shared `apiFetch` helper attaches the Bearer token to every backend call and transparently refreshes it on a 401. One small, well-justified backend addition: a `fake` LLM/embedding provider option (selectable via `LLM_PROVIDER=fake`/`EMBEDDING_PROVIDER=fake`), giving local dev and the Playwright e2e suite a free, deterministic, zero-network way to exercise the send-message path without a real API key.

**Tech Stack:** SvelteKit (SSR, Svelte 5 runes), Tailwind CSS v4 (`@tailwindcss/vite`), `@sveltejs/adapter-node`, Playwright for e2e tests, `pnpm`. Backend: Rust/axum (unchanged except the new fake-provider option).

## Global Constraints

- Frontend lives at `frontend/`, a standalone SvelteKit project — not a monorepo workspace tool (no Turborepo/Nx), just a sibling directory to `backend/`.
- SSR is on (SvelteKit's default) — no `export const ssr = false` anywhere. Auth tokens are httpOnly cookies, never read or stored via client-side JS/localStorage.
- Every backend call from server-side code goes through `apiFetch` (`src/lib/server/api.ts`) except the two unauthenticated calls (`login`, `register` actions), which use the `apiUrl()` helper directly with plain `fetch`.
- `API_URL` is read via `$env/dynamic/private` (never `$env/static/*` — this project's dev/e2e values are set as process env, not compiled in) and is never exposed to client-side code.
- `user_email` cookie is set alongside the auth cookies purely for greeting/display text — never read server-side for any authorization decision, and the backend never sees it.
- Playwright e2e tests are the only test type in this plan (no Vitest/component tests) — matches the approved spec's "small, pragmatic set of e2e tests" scope. Every test generates a unique email per run (`test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`) so tests are safe to re-run against a persistent local Postgres without colliding on `email already registered`.
- The backend's fake LLM/embedding providers are real, production-code additions (`backend/src/llm/fake.rs`, `backend/src/embedding/fake.rs`) — not test-only doubles — selected via `LLM_PROVIDER=fake`/`EMBEDDING_PROVIDER=fake` env vars, so `cargo run` itself can serve the Playwright suite (and any developer's local frontend dev loop) with zero external API dependency. `EMBEDDING_PROVIDER` is optional and defaults to `openai` (preserves existing deployments); `LLM_PROVIDER` remains required with no default, matching the existing convention that nothing security/config-relevant silently defaults.
- No new frontend dependencies beyond what's scaffolded in Task 2 (`@sveltejs/kit`, `svelte`, `tailwindcss`, `@tailwindcss/vite`, `@sveltejs/adapter-node`, `@playwright/test`, `typescript`, `svelte-check`) plus nothing else — no UI component library, no state-management library, no separate HTTP client package (plain `fetch` is enough for 7 endpoints).

## File Structure

- `backend/src/llm/fake.rs`, `backend/src/embedding/fake.rs` — new production fake providers (Task 1).
- `backend/src/llm/config.rs`, `backend/src/embedding/config.rs`, `backend/src/main.rs` — gain the `fake`/`Fake` option (Task 1).
- `backend/tests/embedding_config.rs` — updated for `EmbeddingConfig`'s new required `provider` field (Task 1).
- `frontend/` — new SvelteKit project (Task 2).
  - `src/lib/server/api.ts` — `apiFetch`, `apiUrl` (Task 3).
  - `src/hooks.server.ts`, `src/app.d.ts` — `locals.accessToken` (Task 3).
  - `src/routes/login/`, `src/routes/register/` — public auth pages (Task 3).
  - `src/routes/logout/+server.ts` — logout endpoint (Task 4).
  - `src/routes/(app)/+layout.server.ts`, `+layout.svelte` — authenticated shell + guard + session list (Task 4).
  - `src/lib/components/Sidebar.svelte`, `SessionListItem.svelte` (Task 4).
  - `src/routes/(app)/+page.server.ts`, `+page.svelte` — empty state + New Chat (Task 5).
  - `src/routes/(app)/chat/[sessionId]/+page.server.ts`, `+page.svelte` — conversation view (Task 6).
  - `src/lib/components/MessageBubble.svelte` (Task 6).
  - `e2e/*.e2e.ts` — Playwright tests, one file per task from Task 3 onward.

---

### Task 1: Backend — fake LLM/embedding providers

**Files:**
- Create: `backend/src/llm/fake.rs`
- Create: `backend/src/embedding/fake.rs`
- Modify: `backend/src/llm/mod.rs`, `backend/src/llm/config.rs`
- Modify: `backend/src/embedding/mod.rs`, `backend/src/embedding/config.rs`
- Modify: `backend/src/main.rs`
- Modify: `backend/tests/embedding_config.rs`

**Interfaces:**
- Produces: `ProviderKind::Fake` (existing enum, `backend/src/llm/config.rs`), `EmbeddingProviderKind::{OpenAi, Fake}` (new enum, `backend/src/embedding/config.rs`, re-exported from `embedding::mod`). `EmbeddingConfig` gains a required `provider: EmbeddingProviderKind` field.
- Env vars: `LLM_PROVIDER` gains a `"fake"` value (alongside existing `anthropic`/`openai`/`gemini`). New optional `EMBEDDING_PROVIDER` (`"openai"` default, or `"fake"`). When either is `"fake"`, that provider's `_MODEL_ID`/`_API_KEY` env vars are not read (no longer required).

This task has no failing-test-first cycle in the usual sense — it's additive production code with an existing test file needing a mechanical update for a new required struct field. Steps:

- [ ] **Step 1: Add the fake LLM provider**

```rust
// backend/src/llm/fake.rs
use async_trait::async_trait;

use super::{ContentBlock, LlmError, LlmProvider, LlmRequest, LlmResponse, StopReason};

pub struct FakeLlmProvider;

#[async_trait]
impl LlmProvider for FakeLlmProvider {
    async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse, LlmError> {
        Ok(LlmResponse {
            content: vec![ContentBlock::Text {
                text: "This is a fake response for local development and testing.".to_string(),
            }],
            stop_reason: StopReason::EndTurn,
            input_tokens: 0,
            output_tokens: 0,
        })
    }
}
```

Add `pub mod fake;` to `backend/src/llm/mod.rs`, alongside the other `pub mod` lines (order doesn't matter — add it after `pub mod config;`).

- [ ] **Step 2: Wire `ProviderKind::Fake` into `llm/config.rs`**

```rust
// backend/src/llm/config.rs — FULL FILE REPLACEMENT
use super::anthropic::AnthropicProvider;
use super::fake::FakeLlmProvider;
use super::gemini::GeminiProvider;
use super::openai::OpenAiProvider;
use super::LlmProvider;

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderKind {
    Anthropic,
    OpenAi,
    Gemini,
    Fake,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub provider: ProviderKind,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

pub fn build_provider(config: ModelConfig, http_client: reqwest::Client) -> Box<dyn LlmProvider> {
    match config.provider {
        ProviderKind::Anthropic => {
            let base_url = config.base_url.unwrap_or_else(AnthropicProvider::default_base_url);
            Box::new(AnthropicProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::OpenAi => {
            let base_url = config.base_url.unwrap_or_else(OpenAiProvider::default_base_url);
            Box::new(OpenAiProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::Gemini => {
            let base_url = config.base_url.unwrap_or_else(GeminiProvider::default_base_url);
            Box::new(GeminiProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::Fake => Box::new(FakeLlmProvider),
    }
}
```

- [ ] **Step 3: Add the fake embedding provider**

```rust
// backend/src/embedding/fake.rs
use async_trait::async_trait;

use super::{EmbeddingError, EmbeddingProvider};

pub struct FakeEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for FakeEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Ok(vec![0.0; 1536])
    }
}
```

Add `pub mod fake;` to `backend/src/embedding/mod.rs`, after `pub mod config;`. Also update its re-export line:

```rust
// backend/src/embedding/mod.rs — the existing `pub use config::{...}` line becomes:
pub use config::{build_embedding_provider, EmbeddingConfig, EmbeddingProviderKind};
```

- [ ] **Step 4: Wire `EmbeddingProviderKind` into `embedding/config.rs`**

```rust
// backend/src/embedding/config.rs — FULL FILE REPLACEMENT
use super::fake::FakeEmbeddingProvider;
use super::openai::OpenAiEmbeddingProvider;
use super::EmbeddingProvider;

#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingProviderKind {
    OpenAi,
    Fake,
}

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProviderKind,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

pub fn build_embedding_provider(config: EmbeddingConfig, http_client: reqwest::Client) -> Box<dyn EmbeddingProvider> {
    match config.provider {
        EmbeddingProviderKind::OpenAi => {
            let base_url = config.base_url.unwrap_or_else(OpenAiEmbeddingProvider::default_base_url);
            Box::new(OpenAiEmbeddingProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        EmbeddingProviderKind::Fake => Box::new(FakeEmbeddingProvider),
    }
}
```

- [ ] **Step 5: Update the existing embedding config test for the new required field**

`backend/tests/embedding_config.rs` currently constructs `EmbeddingConfig { model_id, api_key, base_url }` in two tests. Add the import and the new field to both struct literals — no other changes to this file:

```rust
// backend/tests/embedding_config.rs — change only the top import line and the two EmbeddingConfig literals
use nomi_orchestrator::embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProviderKind};
// ... (rest of the file's imports unchanged)

// In both existing tests, add this as the first field of the `EmbeddingConfig { ... }` literal:
    let config = EmbeddingConfig {
        provider: EmbeddingProviderKind::OpenAi,
        model_id: "text-embedding-3-small".to_string(),
        // ...rest of each literal unchanged
```

- [ ] **Step 6: Update `main.rs` to support `LLM_PROVIDER=fake` and `EMBEDDING_PROVIDER=fake`**

```rust
// backend/src/main.rs — FULL FILE REPLACEMENT
use std::sync::Arc;

use nomi_orchestrator::embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind};
use nomi_orchestrator::llm::{build_provider, LlmProvider, ModelConfig, ProviderKind};

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let http_client = reqwest::Client::new();

    let llm_provider_kind = match std::env::var("LLM_PROVIDER").expect("LLM_PROVIDER must be set").as_str() {
        "anthropic" => ProviderKind::Anthropic,
        "openai" => ProviderKind::OpenAi,
        "gemini" => ProviderKind::Gemini,
        "fake" => ProviderKind::Fake,
        other => panic!("unknown LLM_PROVIDER: {other} (expected anthropic, openai, gemini, or fake)"),
    };
    let (llm_model_id, llm_api_key) = match llm_provider_kind {
        ProviderKind::Fake => (String::new(), String::new()),
        _ => (
            std::env::var("LLM_MODEL_ID").expect("LLM_MODEL_ID must be set"),
            std::env::var("LLM_API_KEY").expect("LLM_API_KEY must be set"),
        ),
    };
    let model_config = ModelConfig {
        provider: llm_provider_kind,
        model_id: llm_model_id,
        api_key: llm_api_key,
        base_url: std::env::var("LLM_BASE_URL").ok(),
    };
    let provider: Arc<dyn LlmProvider> = Arc::from(build_provider(model_config, http_client.clone()));

    let embedding_provider_kind = match std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "openai".to_string()).as_str() {
        "openai" => EmbeddingProviderKind::OpenAi,
        "fake" => EmbeddingProviderKind::Fake,
        other => panic!("unknown EMBEDDING_PROVIDER: {other} (expected openai or fake)"),
    };
    let (embedding_model_id, embedding_api_key) = match embedding_provider_kind {
        EmbeddingProviderKind::Fake => (String::new(), String::new()),
        EmbeddingProviderKind::OpenAi => (
            std::env::var("EMBEDDING_MODEL_ID").expect("EMBEDDING_MODEL_ID must be set"),
            std::env::var("EMBEDDING_API_KEY").expect("EMBEDDING_API_KEY must be set"),
        ),
    };
    let embedding_config = EmbeddingConfig {
        provider: embedding_provider_kind,
        model_id: embedding_model_id,
        api_key: embedding_api_key,
        base_url: std::env::var("EMBEDDING_BASE_URL").ok(),
    };
    let embedding_provider: Arc<dyn EmbeddingProvider> =
        Arc::from(build_embedding_provider(embedding_config, http_client));

    let state = nomi_orchestrator::app::AppState { pool, jwt_secret, provider, embedding_provider };
    let app = nomi_orchestrator::app::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to port 8080");
    axum::serve(listener, app).await.expect("server error");
}
```

- [ ] **Step 7: Run the full test suite**

Run: `cd backend && cargo test`
Expected: PASS, 164 tests, 0 failures (no new tests added — this task is additive production code plus a mechanical fixture update to `tests/embedding_config.rs`).

- [ ] **Step 8: Manually verify the fake provider serves real requests**

```bash
cd backend
DATABASE_URL=<your local dev Postgres URL> JWT_SECRET=dev-secret LLM_PROVIDER=fake EMBEDDING_PROVIDER=fake cargo run
```

Expected: the server starts and binds to `0.0.0.0:8080` with no panic (confirms the `Fake` branches skip the `LLM_MODEL_ID`/`LLM_API_KEY`/`EMBEDDING_MODEL_ID`/`EMBEDDING_API_KEY` env var reads entirely). Stop the server (Ctrl-C) once confirmed.

- [ ] **Step 9: Commit**

```bash
git add backend/src/llm/fake.rs backend/src/embedding/fake.rs backend/src/llm/mod.rs backend/src/llm/config.rs backend/src/embedding/mod.rs backend/src/embedding/config.rs backend/src/main.rs backend/tests/embedding_config.rs
git commit -m "feat: add fake LLM/embedding providers for local dev and e2e testing"
```

---

### Task 2: Frontend scaffold

**Files:**
- Create: `frontend/` (entire SvelteKit project via `sv create`)
- Modify: `frontend/vite.config.ts` (swap `adapter-auto` for `adapter-node`)
- Create: `frontend/.env.example`
- Create: `frontend/playwright.config.ts` (extend the scaffolded default with the two-webServer setup used by every later task's e2e tests)

**Interfaces:**
- Produces: a running SvelteKit dev server (`pnpm run dev`), a Tailwind-processed build, and a working Playwright e2e harness that starts both the frontend and backend automatically.

- [ ] **Step 1: Scaffold the project**

From the repo root:

```bash
pnpm dlx sv create frontend --template minimal --types ts --add tailwindcss="plugins:none" playwright --install pnpm --no-download-check
```

This produces a SvelteKit project using Svelte 5 (runes mode), Tailwind CSS v4 wired via `@tailwindcss/vite` (no `tailwind.config.js`/PostCSS needed — `src/routes/layout.css` already contains `@import 'tailwindcss';` and is imported from `+layout.svelte`), and a Playwright setup (`playwright.config.ts`, a demo test at `src/routes/demo/playwright/`).

- [ ] **Step 2: Swap `adapter-auto` for `adapter-node`**

`adapter-auto` can't detect a supported deployment target for a self-hosted Node service (it prints "Could not detect a supported production environment" on build) — swap to the explicit, correct adapter for this deployment shape:

```bash
cd frontend
pnpm add -D @sveltejs/adapter-node
```

```ts
// frontend/vite.config.ts — change only this one import line
import adapter from '@sveltejs/adapter-node';
```

- [ ] **Step 3: Remove the scaffolded demo routes**

Delete `src/routes/demo/` entirely (both the generic demo page and the Playwright demo test) — it's scaffold boilerplate, not part of this app:

```bash
rm -rf src/routes/demo
```

Also replace the placeholder `src/routes/+page.svelte` content — this route will be fully replaced by Task 5's empty-state page, but for this task just confirm the dev server runs (Step 5 below); no need to write real content yet.

- [ ] **Step 4: Add `.env.example`**

```bash
# frontend/.env.example
API_URL=http://localhost:8080
```

- [ ] **Step 5: Verify the dev server boots and Tailwind processes a utility class**

```bash
pnpm run dev &
sleep 2
curl -s http://localhost:5173 | grep -o '<h1[^>]*>Welcome to SvelteKit</h1>'
kill %1
```

Expected: the `curl` output shows the `<h1>` tag (confirms the dev server serves the scaffolded page). This is a placeholder check for this task only — Task 5 replaces this page's content and gets its own e2e test.

- [ ] **Step 6: Set up the two-server Playwright config**

Later tasks' e2e tests need both the backend (with `LLM_PROVIDER=fake`) and the frontend (built + previewed) running. Playwright's `webServer` option accepts an array — configure both here so every later task's tests can just assume the stack is up:

```ts
// frontend/playwright.config.ts — FULL FILE REPLACEMENT
import { defineConfig } from '@playwright/test';
import path from 'node:path';

const backendDir = path.resolve(__dirname, '../backend');

export default defineConfig({
	webServer: [
		{
			command: 'cargo run',
			cwd: backendDir,
			port: 8080,
			timeout: 120_000,
			reuseExistingServer: !process.env.CI,
			env: {
				DATABASE_URL: process.env.DATABASE_URL ?? 'postgres://postgres:postgres@localhost:5432/nomi_dev',
				JWT_SECRET: 'e2e-test-secret-do-not-use-in-prod',
				LLM_PROVIDER: 'fake',
				EMBEDDING_PROVIDER: 'fake',
			},
		},
		{
			command: 'npm run build && npm run preview',
			port: 4173,
			reuseExistingServer: !process.env.CI,
		},
	],
	use: {
		baseURL: 'http://localhost:4173',
	},
	testMatch: '**/*.e2e.{ts,js}',
	testDir: 'e2e',
});
```

This assumes a local Postgres is already running and migrated at the `DATABASE_URL` value (override via an actual `DATABASE_URL` env var if your local setup differs — matches the existing convention the Rust test suite already relies on). Create the `e2e/` directory now (empty for this task — Task 3 adds the first test):

```bash
mkdir -p e2e
```

- [ ] **Step 7: Commit**

```bash
git add frontend/
git commit -m "feat: scaffold SvelteKit frontend (Tailwind, adapter-node, Playwright)"
```

---

### Task 3: Auth — register, login, token refresh

**Files:**
- Create: `frontend/src/lib/server/api.ts`
- Create: `frontend/src/hooks.server.ts`
- Modify: `frontend/src/app.d.ts`
- Create: `frontend/src/routes/login/+page.server.ts`, `+page.svelte`
- Create: `frontend/src/routes/register/+page.server.ts`, `+page.svelte`
- Create: `frontend/e2e/auth.e2e.ts`

**Interfaces:**
- Produces: `apiFetch(fetchFn, cookies, path, init?) -> Promise<Response>` and `apiUrl(path) -> string` (both from `src/lib/server/api.ts`) — every later task's server-side code that calls the backend uses these. `locals.accessToken: string | undefined` (from `hooks.server.ts`), consumed by Task 4's layout guard.

- [ ] **Step 1: Write the failing e2e test**

```ts
// frontend/e2e/auth.e2e.ts
import { expect, test } from '@playwright/test';

function uniqueEmail(): string {
	return `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;
}

test('register with a new account lands on the authenticated shell', async ({ page }) => {
	const email = uniqueEmail();

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();

	await expect(page).toHaveURL('/');
	await expect(page.getByText(email)).toBeVisible();
});

test('registering with an already-used email shows an inline error', async ({ page }) => {
	const email = uniqueEmail();

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await page.goto('/logout');
	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();

	await expect(page).toHaveURL('/register');
	await expect(page.getByText(/already registered/i)).toBeVisible();
});

test('login with wrong password shows an inline error', async ({ page }) => {
	const email = uniqueEmail();

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await page.goto('/logout');
	await page.goto('/login');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('wrong password');
	await page.getByRole('button', { name: 'Log in' }).click();

	await expect(page).toHaveURL('/login');
	await expect(page.getByText(/invalid email or password/i)).toBeVisible();
});
```

Note: `/logout` doesn't exist until Task 4 — for this task, temporarily replace each `await page.goto('/logout');` line with the equivalent manual cookie-clearing so this test is self-contained:

```ts
	await page.context().clearCookies();
```

Use `await page.context().clearCookies();` in place of `await page.goto('/logout');` in both tests above for this task — Task 4 can optionally switch these to the real `/logout` route once it exists, but is not required to (this file is not modified again after this task).

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd frontend && pnpm run test:e2e -- auth.e2e.ts`
Expected: FAIL — `/register` and `/login` routes don't exist yet (404).

- [ ] **Step 3: Add `app.d.ts`'s `locals.accessToken` type**

```ts
// frontend/src/app.d.ts — FULL FILE REPLACEMENT
declare global {
	namespace App {
		interface Locals {
			accessToken?: string;
		}
	}
}

export {};
```

- [ ] **Step 4: Add `hooks.server.ts`**

```ts
// frontend/src/hooks.server.ts
import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	event.locals.accessToken = event.cookies.get('access_token');
	return resolve(event);
};
```

- [ ] **Step 5: Add the `apiFetch`/`apiUrl` helper**

```ts
// frontend/src/lib/server/api.ts
import { redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { Cookies } from '@sveltejs/kit';

const API_URL = env.API_URL ?? 'http://localhost:8080';

export function apiUrl(path: string): string {
	return `${API_URL}${path}`;
}

function withAuth(init: RequestInit | undefined, accessToken: string | undefined): RequestInit {
	return {
		...init,
		headers: {
			'Content-Type': 'application/json',
			...(init?.headers ?? {}),
			...(accessToken ? { Authorization: `Bearer ${accessToken}` } : {}),
		},
	};
}

export async function apiFetch(
	fetchFn: typeof fetch,
	cookies: Cookies,
	path: string,
	init?: RequestInit,
): Promise<Response> {
	const accessToken = cookies.get('access_token');
	const response = await fetchFn(apiUrl(path), withAuth(init, accessToken));

	if (response.status !== 401) {
		return response;
	}

	const refreshToken = cookies.get('refresh_token');
	if (!refreshToken) {
		cookies.delete('access_token', { path: '/' });
		throw redirect(303, '/login');
	}

	const refreshResponse = await fetchFn(apiUrl('/api/auth/refresh'), {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ refresh_token: refreshToken }),
	});

	if (!refreshResponse.ok) {
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });
		throw redirect(303, '/login');
	}

	const { access_token } = (await refreshResponse.json()) as { access_token: string };
	cookies.set('access_token', access_token, { httpOnly: true, path: '/', sameSite: 'lax' });

	return fetchFn(apiUrl(path), withAuth(init, access_token));
}
```

- [ ] **Step 6: Add the register page**

```ts
// frontend/src/routes/register/+page.server.ts
import { fail, redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const email = data.get('email');
		const password = data.get('password');
		const orgName = data.get('orgName');

		if (
			typeof email !== 'string' ||
			typeof password !== 'string' ||
			typeof orgName !== 'string' ||
			!email ||
			!password ||
			!orgName
		) {
			return fail(400, { error: 'Email, password, and organization name are required.' });
		}

		const response = await fetch(apiUrl('/api/auth/register'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email, password, org: { mode: 'create', name: orgName } }),
		});

		if (response.status === 409) {
			return fail(409, { error: 'That email is already registered.' });
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Registration failed. Please try again.' });
		}

		const { access_token, refresh_token } = (await response.json()) as {
			access_token: string;
			refresh_token: string;
		};

		cookies.set('access_token', access_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('refresh_token', refresh_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('user_email', email, { httpOnly: false, path: '/', sameSite: 'lax' });

		throw redirect(303, '/');
	},
};
```

```svelte
<!-- frontend/src/routes/register/+page.svelte -->
<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();
</script>

<div class="flex min-h-screen items-center justify-center bg-neutral-50">
	<form
		method="POST"
		use:enhance
		class="w-full max-w-sm space-y-4 rounded-2xl border border-neutral-200 bg-white p-8 shadow-sm"
	>
		<h1 class="text-2xl font-semibold">Create your account</h1>
		{#if form?.error}
			<p class="text-sm text-red-600">{form.error}</p>
		{/if}
		<div>
			<label for="email" class="block text-sm font-medium text-neutral-700">Email</label>
			<input
				id="email"
				name="email"
				type="email"
				required
				class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			/>
		</div>
		<div>
			<label for="password" class="block text-sm font-medium text-neutral-700">Password</label>
			<input
				id="password"
				name="password"
				type="password"
				required
				class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			/>
		</div>
		<div>
			<label for="orgName" class="block text-sm font-medium text-neutral-700">Organization name</label>
			<input
				id="orgName"
				name="orgName"
				type="text"
				required
				class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			/>
		</div>
		<button
			type="submit"
			class="w-full rounded-lg bg-neutral-900 px-4 py-2 font-medium text-white hover:bg-neutral-800"
		>
			Register
		</button>
		<p class="text-center text-sm text-neutral-500">
			Already have an account? <a href="/login" class="underline">Log in</a>
		</p>
	</form>
</div>
```

- [ ] **Step 7: Add the login page**

```ts
// frontend/src/routes/login/+page.server.ts
import { fail, redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const email = data.get('email');
		const password = data.get('password');

		if (typeof email !== 'string' || typeof password !== 'string' || !email || !password) {
			return fail(400, { error: 'Email and password are required.' });
		}

		const response = await fetch(apiUrl('/api/auth/login'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email, password }),
		});

		if (response.status === 401) {
			return fail(401, { error: 'Invalid email or password.' });
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Login failed. Please try again.' });
		}

		const { access_token, refresh_token } = (await response.json()) as {
			access_token: string;
			refresh_token: string;
		};

		cookies.set('access_token', access_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('refresh_token', refresh_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('user_email', email, { httpOnly: false, path: '/', sameSite: 'lax' });

		throw redirect(303, '/');
	},
};
```

```svelte
<!-- frontend/src/routes/login/+page.svelte -->
<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();
</script>

<div class="flex min-h-screen items-center justify-center bg-neutral-50">
	<form
		method="POST"
		use:enhance
		class="w-full max-w-sm space-y-4 rounded-2xl border border-neutral-200 bg-white p-8 shadow-sm"
	>
		<h1 class="text-2xl font-semibold">Welcome back</h1>
		{#if form?.error}
			<p class="text-sm text-red-600">{form.error}</p>
		{/if}
		<div>
			<label for="email" class="block text-sm font-medium text-neutral-700">Email</label>
			<input
				id="email"
				name="email"
				type="email"
				required
				class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			/>
		</div>
		<div>
			<label for="password" class="block text-sm font-medium text-neutral-700">Password</label>
			<input
				id="password"
				name="password"
				type="password"
				required
				class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			/>
		</div>
		<button
			type="submit"
			class="w-full rounded-lg bg-neutral-900 px-4 py-2 font-medium text-white hover:bg-neutral-800"
		>
			Log in
		</button>
		<p class="text-center text-sm text-neutral-500">
			No account? <a href="/register" class="underline">Register</a>
		</p>
	</form>
</div>
```

Since `/` (the authenticated shell) doesn't exist until Task 4, temporarily add a minimal placeholder so the e2e test's `page.getByText(email)` assertion has something to find — read the `user_email` cookie directly in a temporary `+page.server.ts` for this task only:

```ts
// frontend/src/routes/+page.server.ts — temporary, REPLACED ENTIRELY by Task 4's version
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies }) => {
	return { userEmail: cookies.get('user_email') ?? '' };
};
```

```svelte
<!-- frontend/src/routes/+page.svelte — temporary, REPLACED ENTIRELY by Task 4's version -->
<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<p>{data.userEmail}</p>
```

- [ ] **Step 8: Run the test to verify it passes**

Run: `cd frontend && pnpm run test:e2e -- auth.e2e.ts`
Expected: PASS (3 tests).

- [ ] **Step 9: Commit**

```bash
git add frontend/src/app.d.ts frontend/src/hooks.server.ts frontend/src/lib/server/api.ts frontend/src/routes/login frontend/src/routes/register frontend/src/routes/+page.server.ts frontend/src/routes/+page.svelte frontend/e2e/auth.e2e.ts
git commit -m "feat: add register/login pages with httpOnly cookie auth and token refresh"
```

---

### Task 4: Authenticated app shell (route guard, session list, sidebar, logout)

**Files:**
- Create: `frontend/src/lib/types.ts`
- Create: `frontend/src/routes/(app)/+layout.server.ts`, `+layout.svelte`
- Create: `frontend/src/lib/components/Sidebar.svelte`, `SessionListItem.svelte`
- Create: `frontend/src/routes/logout/+server.ts`
- Delete: `frontend/src/routes/+page.server.ts`, `+page.svelte` (Task 3's temporary placeholder — this task moves the index page inside the `(app)` route group)
- Create: `frontend/e2e/shell.e2e.ts`

**Interfaces:**
- Consumes: `apiFetch` (Task 3).
- Produces: `data.sessions` and `data.userEmail`, available to every page under the `(app)` group (Task 5's empty state and Task 6's conversation view both read `data.userEmail`; Task 5 also reads `data.sessions` implicitly via the shared layout).

- [ ] **Step 1: Write the failing e2e test**

```ts
// frontend/e2e/shell.e2e.ts
import { expect, test } from '@playwright/test';

test('visiting the home page while logged out redirects to login', async ({ page }) => {
	await page.goto('/');
	await expect(page).toHaveURL('/login');
});

test('after logging in, the sidebar and logout are visible; logout returns to login', async ({ page }) => {
	const email = `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();

	await expect(page).toHaveURL('/');
	await expect(page.getByRole('button', { name: '+ New Chat' })).toBeVisible();
	await expect(page.getByText(email)).toBeVisible();

	await page.getByRole('button', { name: 'Log out' }).click();
	await expect(page).toHaveURL('/login');

	await page.goto('/');
	await expect(page).toHaveURL('/login');
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd frontend && pnpm run test:e2e -- shell.e2e.ts`
Expected: FAIL — no redirect exists yet for a logged-out visit to `/`, and no Sidebar/logout button exist.

- [ ] **Step 3: Remove Task 3's temporary index page**

```bash
cd frontend
rm src/routes/+page.server.ts src/routes/+page.svelte
mkdir -p "src/routes/(app)"
```

- [ ] **Step 4: Add the shared `$lib/types.ts`**

Shared response-shape types live in a plain `$lib` module — never imported directly from a `.server.ts` file, so components can safely import them without any risk of pulling server-only code into client bundles:

```ts
// frontend/src/lib/types.ts
export interface SessionSummary {
	id: string;
	channel: string;
	chat_type: string;
	chat_id: string;
	last_message: { content: string; created_at: string } | null;
	agent_active: boolean;
	updated_at: string;
}
```

- [ ] **Step 5: Add the `(app)` layout guard + session list load**

```ts
// frontend/src/routes/(app)/+layout.server.ts
import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { SessionSummary } from '$lib/types';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/login');
	}

	const response = await apiFetch(fetch, cookies, '/api/sessions');
	if (!response.ok) {
		throw redirect(303, '/login');
	}

	const { sessions } = (await response.json()) as { sessions: SessionSummary[] };
	const userEmail = cookies.get('user_email') ?? '';

	return { sessions, userEmail };
};
```

```svelte
<!-- frontend/src/routes/(app)/+layout.svelte -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: Snippet } = $props();
</script>

<div class="flex h-screen bg-neutral-50">
	<Sidebar sessions={data.sessions} userEmail={data.userEmail} />
	<main class="flex-1 overflow-hidden">
		{@render children()}
	</main>
</div>
```

- [ ] **Step 6: Add `SessionListItem.svelte`**

```svelte
<!-- frontend/src/lib/components/SessionListItem.svelte -->
<script lang="ts">
	import type { SessionSummary } from '$lib/types';

	let { session }: { session: SessionSummary } = $props();

	function timeAgo(iso: string): string {
		const diffMs = Date.now() - new Date(iso).getTime();
		const minutes = Math.floor(diffMs / 60000);
		if (minutes < 1) return 'just now';
		if (minutes < 60) return `${minutes}m`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h`;
		return `${Math.floor(hours / 24)}d`;
	}
</script>

<a
	href={`/chat/${session.id}`}
	class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm hover:bg-neutral-100"
>
	<span class="relative flex h-2 w-2 shrink-0">
		{#if session.agent_active}
			<span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-amber-400 opacity-75"></span>
			<span class="relative inline-flex h-2 w-2 rounded-full bg-amber-500"></span>
		{/if}
	</span>
	<span class="flex-1 truncate text-neutral-700">
		{session.last_message?.content ?? 'New chat'}
	</span>
	<span class="shrink-0 text-xs text-neutral-400">{timeAgo(session.updated_at)}</span>
</a>
```

- [ ] **Step 7: Add `Sidebar.svelte`**

```svelte
<!-- frontend/src/lib/components/Sidebar.svelte -->
<script lang="ts">
	import { enhance } from '$app/forms';
	import SessionListItem from './SessionListItem.svelte';
	import type { SessionSummary } from '$lib/types';

	let { sessions, userEmail }: { sessions: SessionSummary[]; userEmail: string } = $props();
</script>

<aside class="flex w-72 flex-col border-r border-neutral-200 bg-white">
	<div class="flex items-center gap-2 border-b border-neutral-200 px-4 py-4">
		<span class="text-lg font-semibold">Nomi</span>
	</div>

	<form method="POST" action="/?/newChat" use:enhance class="px-3 pt-3">
		<button
			type="submit"
			class="w-full rounded-lg bg-neutral-900 px-4 py-2 text-sm font-medium text-white hover:bg-neutral-800"
		>
			+ New Chat
		</button>
	</form>

	<nav class="flex-1 space-y-1 overflow-y-auto px-3 py-3">
		{#each sessions as session (session.id)}
			<SessionListItem {session} />
		{/each}
	</nav>

	<form method="POST" action="/logout" class="flex items-center justify-between border-t border-neutral-200 px-4 py-3">
		<span class="truncate text-sm text-neutral-600">{userEmail}</span>
		<button type="submit" class="text-sm text-neutral-400 hover:text-neutral-700">Log out</button>
	</form>
</aside>
```

- [ ] **Step 8: Add the logout endpoint**

```ts
// frontend/src/routes/logout/+server.ts
import { redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ cookies, fetch }) => {
	const refreshToken = cookies.get('refresh_token');
	if (refreshToken) {
		await fetch(apiUrl('/api/auth/logout'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ refresh_token: refreshToken }),
		});
	}
	cookies.delete('access_token', { path: '/' });
	cookies.delete('refresh_token', { path: '/' });
	cookies.delete('user_email', { path: '/' });
	throw redirect(303, '/login');
};
```

- [ ] **Step 9: Add a minimal `(app)/+page.svelte` so the route resolves for this task's test**

Task 5 replaces this file's content entirely with the real empty state — for this task, just enough to satisfy the e2e test's assertions (Sidebar renders via the layout regardless of this page's content):

```svelte
<!-- frontend/src/routes/(app)/+page.svelte — placeholder, REPLACED ENTIRELY by Task 5 -->
<div></div>
```

Also add a placeholder `newChat` action so Sidebar's form target resolves (Task 5 replaces this with the real implementation):

```ts
// frontend/src/routes/(app)/+page.server.ts — placeholder, REPLACED ENTIRELY by Task 5
import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions: Actions = {
	newChat: async () => {
		return fail(501, { error: 'not implemented yet' });
	},
};
```

- [ ] **Step 10: Run the test to verify it passes**

Run: `cd frontend && pnpm run test:e2e -- shell.e2e.ts`
Expected: PASS (2 tests).

- [ ] **Step 11: Commit**

```bash
git add frontend/src/lib/types.ts "frontend/src/routes/(app)" frontend/src/lib/components/Sidebar.svelte frontend/src/lib/components/SessionListItem.svelte frontend/src/routes/logout frontend/e2e/shell.e2e.ts
git commit -m "feat: add authenticated app shell with route guard, sidebar, and logout"
```

---

### Task 5: Empty state + New Chat

**Files:**
- Modify: `frontend/src/routes/(app)/+page.server.ts`, `+page.svelte` (replace Task 4's placeholders entirely)
- Create: `frontend/e2e/new-chat.e2e.ts`

**Interfaces:**
- Consumes: `apiFetch` (Task 3), `data.userEmail` (Task 4's layout load).
- Produces: the `newChat` form action other tasks' Sidebar form already targets (Task 4 already wired the form's `action="/?/newChat"` — this task provides the real implementation).

- [ ] **Step 1: Write the failing e2e test**

```ts
// frontend/e2e/new-chat.e2e.ts
import { expect, test } from '@playwright/test';

test('the empty state shows a greeting and starting a new chat navigates to it', async ({ page }) => {
	const email = `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await expect(page.getByRole('heading', { name: `Hi, ${email}!` })).toBeVisible();

	await page.getByRole('button', { name: 'New Chat' }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]+/);
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd frontend && pnpm run test:e2e -- new-chat.e2e.ts`
Expected: FAIL — the placeholder `(app)/+page.svelte` has no greeting or "New Chat" button, and the placeholder action returns a 501.

- [ ] **Step 3: Replace the placeholder action with the real `newChat` implementation**

```ts
// frontend/src/routes/(app)/+page.server.ts — FULL FILE REPLACEMENT
import { fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Actions } from './$types';

export const actions: Actions = {
	newChat: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/sessions', { method: 'POST' });
		if (!response.ok) {
			return fail(500, { error: 'Could not start a new chat.' });
		}
		const { session_id } = (await response.json()) as { session_id: string };
		throw redirect(303, `/chat/${session_id}`);
	},
};
```

- [ ] **Step 4: Replace the placeholder empty-state page**

```svelte
<!-- frontend/src/routes/(app)/+page.svelte — FULL FILE REPLACEMENT -->
<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<div class="flex h-full flex-col items-center justify-center gap-4 text-center">
	<h1 class="text-3xl font-semibold text-neutral-900">Hi, {data.userEmail}!</h1>
	<p class="text-lg text-neutral-500">How can I assist you today?</p>
	<form method="POST" action="?/newChat" use:enhance>
		<button
			type="submit"
			class="rounded-lg bg-neutral-900 px-6 py-3 font-medium text-white hover:bg-neutral-800"
		>
			New Chat
		</button>
	</form>
</div>
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cd frontend && pnpm run test:e2e -- new-chat.e2e.ts`
Expected: PASS. Note this will still fail with a 404 on the `/chat/[sessionId]` destination page until Task 6 exists — for this task, the assertion only checks the URL pattern (`await expect(page).toHaveURL(/\/chat\/[0-9a-f-]+/)`), not the destination page's content, so a 404 page at that URL still satisfies the URL-pattern assertion. Confirm this is genuinely the case by running the test now.

- [ ] **Step 6: Commit**

```bash
git add "frontend/src/routes/(app)/+page.server.ts" "frontend/src/routes/(app)/+page.svelte" frontend/e2e/new-chat.e2e.ts
git commit -m "feat: add empty state and New Chat flow"
```

---

### Task 6: Conversation view (message history + send)

**Files:**
- Modify: `frontend/src/lib/types.ts` (add `MessageItem`)
- Create: `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts`, `+page.svelte`
- Create: `frontend/src/lib/components/MessageBubble.svelte`
- Create: `frontend/e2e/conversation.e2e.ts`

**Interfaces:**
- Consumes: `apiFetch` (Task 3).
- Produces: nothing consumed by later tasks (this is the last task in the plan).

- [ ] **Step 1: Write the failing e2e tests**

```ts
// frontend/e2e/conversation.e2e.ts
import { expect, test } from '@playwright/test';

async function registerAndStartChat(page: import('@playwright/test').Page): Promise<string> {
	const email = `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await page.getByRole('button', { name: 'New Chat' }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]+/);

	return email;
}

test('sending a message shows the user bubble and the fake assistant reply', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('hello there');
	await page.getByRole('button', { name: 'Send' }).click();

	await expect(page.getByText('hello there')).toBeVisible();
	await expect(page.getByText('This is a fake response for local development and testing.')).toBeVisible();
});

test('a session history persists across a reload', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('remember this');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.getByText('remember this')).toBeVisible();

	await page.reload();
	await expect(page.getByText('remember this')).toBeVisible();
	await expect(page.getByText('This is a fake response for local development and testing.')).toBeVisible();
});

test('visiting a nonexistent session redirects to the empty state', async ({ page }) => {
	await registerAndStartChat(page);

	await page.goto('/chat/00000000-0000-0000-0000-000000000000');
	await expect(page).toHaveURL('/');
});

test('a backend 502 on send shows an inline "no reply yet" note without losing the sent text', async ({ page }) => {
	await registerAndStartChat(page);

	await page.route('**/api/sessions/*/messages', async (route) => {
		if (route.request().method() === 'POST') {
			await route.fulfill({ status: 502, contentType: 'application/json', body: JSON.stringify({ error: 'turn failed' }) });
		} else {
			await route.continue();
		}
	});

	await page.getByPlaceholder('Ask me anything...').fill('this will fail');
	await page.getByRole('button', { name: 'Send' }).click();

	await expect(page.getByText('this will fail')).toBeVisible();
	await expect(page.getByText(/no reply yet/i)).toBeVisible();
});
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd frontend && pnpm run test:e2e -- conversation.e2e.ts`
Expected: FAIL — `/chat/[sessionId]` doesn't exist yet (404 for all four tests; note the third test currently "passes" by accident since a 404 page isn't `/`, so it actually fails the `toHaveURL('/')` assertion too — confirm all four genuinely fail before implementing).

- [ ] **Step 3: Add `MessageItem` to `$lib/types.ts` and add `MessageBubble.svelte`**

```ts
// frontend/src/lib/types.ts — APPEND to the existing file (SessionSummary stays as-is)
export interface MessageItem {
	id: string;
	sender: 'user' | 'assistant';
	content: string;
	created_at: string;
}
```

```svelte
<!-- frontend/src/lib/components/MessageBubble.svelte -->
<script lang="ts">
	import type { MessageItem } from '$lib/types';

	let { message }: { message: MessageItem } = $props();
</script>

<div class="flex {message.sender === 'user' ? 'justify-end' : 'justify-start'}">
	<div
		class="max-w-md rounded-2xl px-4 py-2 {message.sender === 'user'
			? 'bg-neutral-900 text-white'
			: 'bg-neutral-100 text-neutral-900'}"
	>
		{message.content}
	</div>
</div>
```

- [ ] **Step 4: Add the conversation view's server logic**

```ts
// frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts
import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { MessageItem } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`);

	if (response.status === 404) {
		throw redirect(303, '/');
	}
	if (!response.ok) {
		throw error(response.status, 'Could not load this chat.');
	}

	const { messages } = (await response.json()) as { messages: MessageItem[] };
	return { messages };
};

export const actions: Actions = {
	default: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const text = data.get('text');

		if (typeof text !== 'string' || !text.trim()) {
			return fail(400, { error: 'Message cannot be empty.' });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`, {
			method: 'POST',
			body: JSON.stringify({ text }),
		});

		if (response.status === 502) {
			return fail(502, { turnFailed: true, sentText: text });
		}
		if (response.status === 404) {
			throw redirect(303, '/');
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to send message.' });
		}

		const { user_message, assistant_message } = (await response.json()) as {
			user_message: MessageItem;
			assistant_message: MessageItem;
		};

		return { user_message, assistant_message };
	},
};
```

- [ ] **Step 5: Add the conversation view's page**

```svelte
<!-- frontend/src/routes/(app)/chat/[sessionId]/+page.svelte -->
<script lang="ts">
	import { enhance } from '$app/forms';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let messages = $derived.by(() => {
		const base = [...data.messages];
		if (form?.user_message && form?.assistant_message) {
			base.push(form.user_message, form.assistant_message);
		}
		return base;
	});
</script>

<div class="flex h-full flex-col">
	<div class="flex-1 space-y-4 overflow-y-auto px-6 py-6">
		{#each messages as message (message.id)}
			<MessageBubble {message} />
		{/each}
		{#if form?.turnFailed}
			<div class="flex justify-end">
				<div class="max-w-md rounded-2xl bg-neutral-900 px-4 py-2 text-white">{form.sentText}</div>
			</div>
			<p class="text-right text-sm text-neutral-400">No reply yet — try sending again.</p>
		{/if}
		{#if form?.error}
			<p class="text-center text-sm text-red-600">{form.error}</p>
		{/if}
	</div>

	<form method="POST" use:enhance={() => {
		return async ({ update }) => {
			await update({ reset: true });
		};
	}} class="border-t border-neutral-200 bg-white px-6 py-4">
		<div class="flex items-center gap-2 rounded-full border border-neutral-300 px-4 py-2">
			<input
				name="text"
				type="text"
				placeholder="Ask me anything..."
				required
				class="flex-1 border-none bg-transparent outline-none"
			/>
			<button
				type="submit"
				class="rounded-full bg-neutral-900 px-4 py-1.5 text-sm font-medium text-white hover:bg-neutral-800"
			>
				Send
			</button>
		</div>
	</form>
</div>
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cd frontend && pnpm run test:e2e -- conversation.e2e.ts`
Expected: PASS (4 tests).

- [ ] **Step 7: Run the full e2e suite and type-check**

Run: `cd frontend && pnpm run test:e2e && pnpm run check`
Expected: all e2e test files pass (auth, shell, new-chat, conversation), and `svelte-check` reports no type errors.

- [ ] **Step 8: Commit**

```bash
git add frontend/src/lib/types.ts "frontend/src/routes/(app)/chat" frontend/src/lib/components/MessageBubble.svelte frontend/e2e/conversation.e2e.ts
git commit -m "feat: add conversation view with message history and send"
```
