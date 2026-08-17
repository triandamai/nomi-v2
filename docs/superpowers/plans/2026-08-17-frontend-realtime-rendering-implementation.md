# Frontend Realtime Rendering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the chat page to the backend's live WebSocket bridge — SvelteKit relays `GET /api/sessions/:id/ws` to the browser over a real WebSocket, and the chat page shows a typing indicator while a reply is generating, then renders the persisted reply once it's done — and fix the chat page's send flow, which is currently broken against the ingest-only `POST /messages` contract.

**Architecture:** A framework-agnostic Node module (`ws-proxy/session-stream-proxy.js`) implements the whole browser↔SvelteKit↔Rust relay — path matching, cookie-to-Bearer auth forwarding, verbatim frame piping, and upstream-reconnect-with-backoff — and is wired unmodified into all three run modes that need it (`vite dev`, `vite preview` — which is what the existing Playwright e2e harness boots — and true production via a new `server.js`). The browser opens a plain `WebSocket` to `/chat/{sessionId}/ws` and tracks only a boolean "reply in flight" state; the finished reply always comes from a REST re-fetch (`invalidateAll()`), never from client-accumulated stream text.

**Tech Stack:** `ws` (Node WebSocket server+client) and `cookie` (upgrade-request cookie parsing) as new runtime deps; `vitest` + `@types/ws` as new dev deps, scoped to unit-testing the proxy module only — this project has no Vitest today (Playwright only).

## Global Constraints

- Both legs (browser↔SvelteKit and SvelteKit↔Rust) are real WebSocket connections, not Server-Sent Events.
- No JWT verification happens in Node. The `access_token` cookie is forwarded verbatim as `Authorization: Bearer <token>` to Rust; Rust's existing `authorize_session_access` is the sole auth authority.
- Rejection mapping: Rust 401 → browser WS close code `4401`; Rust 404 → close code `4404`. Both are terminal — the browser must stop retrying on these, not backoff-retry forever. Any other upstream failure is transient.
- The SvelteKit↔Rust leg reconnects independently of the browser↔SvelteKit leg (the browser socket stays open across a Rust-side blip). If upstream retries exceed ~60s of backoff, the proxy gives up and closes the browser socket with code `1011`, letting the browser's own (non-terminal) reconnect loop restart clean.
- The proxy never parses `StreamEnvelope`/`StreamEvent` JSON — it forwards text frames verbatim in both directions.
- The browser never accumulates streamed delta text. It tracks only a boolean `pendingReply` state; the actual reply content always comes from re-fetching `GET /api/sessions/:id/messages` (via SvelteKit's `invalidateAll()`) once `TurnCompleted` arrives.
- `TurnFailed` shows a generic inline error in the UI — never the envelope's raw `error` string.
- The proxy logic lives in exactly one module (`ws-proxy/session-stream-proxy.js`), reused unmodified by all three entrypoints (`server.js`, `vite dev`, `vite preview`).

---

## File Structure

- Create `frontend/ws-proxy/session-stream-proxy.js` — the shared relay logic (plain ESM JS, not TypeScript: it must run identically under plain `node server.js` in production and under Vite's Node-side plugin hooks in dev/preview, neither of which goes through SvelteKit's `src/lib`/`$lib` build pipeline that compiles `.ts`).
- Create `frontend/ws-proxy/session-stream-proxy.test.js` — Vitest unit tests for the module above.
- Create `frontend/vitest.config.ts` — a minimal Vitest config scoped to `ws-proxy/**/*.test.js`, kept separate from the main `vite.config.ts` (which carries the SvelteKit plugin) to avoid entangling a Node-only test target with the app's SSR/client build pipeline.
- Create `frontend/server.js` — the new production entrypoint (`node server.js`, replacing plain `node build`): wraps `adapter-node`'s built `handler` in a plain `http.Server` and attaches the proxy.
- Create `frontend/vite-plugins/dev-ws-proxy.ts` — a Vite plugin attaching the same proxy to `vite dev` (`configureServer`) and `vite preview` (`configurePreviewServer` — this is the hook the existing Playwright e2e harness exercises, since `npm run preview` is how `playwright.config.ts` boots the frontend).
- Modify `frontend/vite.config.ts` — register the new plugin.
- Modify `frontend/package.json` — add `ws`, `cookie` (runtime), `vitest`, `@types/ws` (dev); add `start` and `test:unit` scripts.
- Modify `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts` — drop the dead `assistant_message`/502 handling now that `POST /messages` is ingest-only.
- Modify `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte` — add the browser WebSocket client, `pendingReply`/`turnError`/`connectionLost` state, and their UI.
- Modify `frontend/e2e/conversation.e2e.ts` — fix two tests that assumed a synchronous assistant reply (now async, over WS), replace the stale "502 shows no reply yet" test (that status code no longer exists) with a `TurnFailed`-driven inline-error test using the fake LLM provider's existing `__SIMULATE_TURN_FAILURE__` sentinel (`backend/src/llm/fake.rs`), and add a same-connection second-turn test proving the session-scoped (not per-turn) lifecycle in a real browser.

**Operational note carried into Task 4, not a code change:** these e2e tests require `cargo run --bin worker` (`backend/src/bin/worker.rs`) running alongside the backend server and EMQX (`backend/docker-compose.yml`) — the worker is what actually processes queued turns and publishes the MQTT events this whole feature relays. `playwright.config.ts`'s `webServer` array is deliberately **not** extended to auto-start it: Playwright's `webServer` entries need a `port` or `url` to know when a process is "ready" (confirmed against Playwright's docs), and the worker binary exposes neither — it's a background job processor with no listening port. Wiring it in without a real readiness signal would be unreliable config, not a shortcut. Start it manually (or add your own CI orchestration) before running `npm run test:e2e`.

---

### Task 1: The shared WebSocket proxy module

**Files:**
- Create: `frontend/ws-proxy/session-stream-proxy.js`
- Create: `frontend/ws-proxy/session-stream-proxy.test.js`
- Create: `frontend/vitest.config.ts`
- Modify: `frontend/package.json`

**Interfaces:**
- Produces: `export function attachSessionStreamProxy(server, options = {})` — `server` is a Node `http.Server` (or `null`/`undefined`, in which case it's a no-op — covers Vite's middleware mode where `server.httpServer` can be `null`). `options` may include `apiUrl` (override for `process.env.API_URL`), `initialRetryDelayMs`, `maxRetryDelayMs`, `maxTotalRetryMs` (all optional, defaulting to `1000`/`30000`/`60000`) — Task 2's entrypoints call this with no options; this task's own tests use the overrides to keep retry timing fast.

- [ ] **Step 1: Add the new dependencies**

Edit `frontend/package.json`. Add to `dependencies` (create the key if it doesn't exist — currently this project only has `devDependencies`):

```json
"dependencies": {
	"cookie": "^2.0.1",
	"ws": "^8.21.3"
}
```

Add to `devDependencies`:

```json
"@types/ws": "^8.18.1",
"vitest": "^4.1.10"
```

Add to `scripts`:

```json
"start": "node server.js",
"test:unit": "vitest run"
```

Run: `cd frontend && npm install`
Expected: installs cleanly, `node_modules/ws` and `node_modules/cookie` present.

- [ ] **Step 2: Add the Vitest config**

Create `frontend/vitest.config.ts`:

```ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
	test: {
		include: ['ws-proxy/**/*.test.js'],
		environment: 'node'
	}
});
```

- [ ] **Step 3: Write the failing tests**

Create `frontend/ws-proxy/session-stream-proxy.test.js`:

```js
import { describe, it, expect, afterEach } from 'vitest';
import { createServer } from 'node:http';
import { WebSocketServer, WebSocket } from 'ws';
import { attachSessionStreamProxy } from './session-stream-proxy.js';

const SESSION_ID = '11111111-1111-1111-1111-111111111111';

/** Starts a bare http.Server with the proxy attached, listening on an ephemeral port. */
async function startProxyServer(options) {
	const server = createServer((_req, res) => {
		res.statusCode = 404;
		res.end();
	});
	attachSessionStreamProxy(server, options);
	await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
	const { port } = server.address();
	return { server, port };
}

/** A fake "Rust backend" WS server. Optionally rejects the handshake with a fixed status. */
async function startFakeUpstream({ rejectStatus, port: fixedPort = 0 } = {}) {
	const server = createServer((_req, res) => {
		if (rejectStatus) {
			res.statusCode = rejectStatus;
			res.end();
		}
	});
	const wss = new WebSocketServer({ noServer: true });
	if (!rejectStatus) {
		server.on('upgrade', (request, socket, head) => {
			wss.handleUpgrade(request, socket, head, (ws) => wss.emit('connection', ws, request));
		});
	}
	await new Promise((resolve, reject) => {
		server.once('error', reject);
		server.listen(fixedPort, '127.0.0.1', resolve);
	});
	const { port } = server.address();
	return { server, wss, port, url: `http://127.0.0.1:${port}` };
}

function connectBrowserClient(port, sessionId = SESSION_ID, cookie = 'access_token=test-token') {
	return new WebSocket(`ws://127.0.0.1:${port}/chat/${sessionId}/ws`, { headers: { cookie } });
}

function waitFor(ws, event) {
	return new Promise((resolve) => ws.once(event, (...args) => resolve(args)));
}

describe('session-stream-proxy', () => {
	let cleanups = [];
	afterEach(async () => {
		await Promise.all(cleanups.map((fn) => fn()));
		cleanups = [];
	});

	it('relays messages verbatim in both directions once the upstream is open', async () => {
		const upstream = await startFakeUpstream();
		cleanups.push(() => upstream.server.close());
		upstream.wss.on('connection', (ws) => ws.on('message', (data) => ws.send(`echo:${data}`)));

		const proxy = await startProxyServer({ apiUrl: upstream.url });
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port);
		cleanups.push(() => browser.close());
		await waitFor(browser, 'open');

		browser.send('hello');
		const [reply] = await waitFor(browser, 'message');
		expect(reply.toString()).toBe('echo:hello');
	});

	it('maps a 401 from the upstream to close code 4401', async () => {
		const upstream = await startFakeUpstream({ rejectStatus: 401 });
		cleanups.push(() => upstream.server.close());
		const proxy = await startProxyServer({ apiUrl: upstream.url });
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port);
		cleanups.push(() => browser.close());
		const [code] = await waitFor(browser, 'close');
		expect(code).toBe(4401);
	});

	it('maps a 404 from the upstream to close code 4404', async () => {
		const upstream = await startFakeUpstream({ rejectStatus: 404 });
		cleanups.push(() => upstream.server.close());
		const proxy = await startProxyServer({ apiUrl: upstream.url });
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port);
		cleanups.push(() => browser.close());
		const [code] = await waitFor(browser, 'close');
		expect(code).toBe(4404);
	});

	it('closes with 1011 when the upstream is unreachable on the first attempt', async () => {
		const proxy = await startProxyServer({ apiUrl: 'http://127.0.0.1:1' });
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port);
		cleanups.push(() => browser.close());
		const [code] = await waitFor(browser, 'close');
		expect(code).toBe(1011);
	});

	it('reconnects the upstream leg and keeps relaying without dropping the browser socket', async () => {
		const first = await startFakeUpstream();
		cleanups.push(() => first.server.close());
		first.wss.on('connection', (ws) => ws.on('message', (data) => ws.send(`first:${data}`)));

		const proxy = await startProxyServer({
			apiUrl: first.url,
			initialRetryDelayMs: 20,
			maxRetryDelayMs: 50,
			maxTotalRetryMs: 2000
		});
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port);
		cleanups.push(() => browser.close());
		await waitFor(browser, 'open');

		browser.send('one');
		const [firstReply] = await waitFor(browser, 'message');
		expect(firstReply.toString()).toBe('first:one');

		const firstPort = first.port;
		await new Promise((resolve) => first.server.close(resolve));

		const second = await startFakeUpstream({ port: firstPort });
		cleanups.push(() => second.server.close());
		second.wss.on('connection', (ws) => ws.on('message', (data) => ws.send(`second:${data}`)));

		// Long enough for the 20ms/50ms-capped backoff to reconnect.
		await new Promise((resolve) => setTimeout(resolve, 300));

		browser.send('two');
		const [secondReply] = await waitFor(browser, 'message');
		expect(secondReply.toString()).toBe('second:two');
		expect(browser.readyState).toBe(WebSocket.OPEN);
	});

	it('gives up and closes the browser socket with 1011 if the upstream never comes back', async () => {
		const first = await startFakeUpstream();
		cleanups.push(() => first.server.close());
		first.wss.on('connection', () => {});

		const proxy = await startProxyServer({
			apiUrl: first.url,
			initialRetryDelayMs: 20,
			maxRetryDelayMs: 30,
			maxTotalRetryMs: 150
		});
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port);
		cleanups.push(() => browser.close());
		await waitFor(browser, 'open');

		await new Promise((resolve) => first.server.close(resolve));
		// Nothing is listening on first.port again from here on.

		const [code] = await waitFor(browser, 'close');
		expect(code).toBe(1011);
	});
});
```

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cd frontend && npm run test:unit`
Expected: fails to resolve `./session-stream-proxy.js` (module doesn't exist yet).

- [ ] **Step 5: Write the implementation**

Create `frontend/ws-proxy/session-stream-proxy.js`:

```js
import { WebSocketServer, WebSocket } from 'ws';
import { parse as parseCookie } from 'cookie';

const SESSION_WS_PATH =
	/^\/chat\/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\/ws$/;

const DEFAULT_INITIAL_RETRY_DELAY_MS = 1000;
const DEFAULT_MAX_RETRY_DELAY_MS = 30000;
const DEFAULT_MAX_TOTAL_RETRY_MS = 60000;

function resolveApiUrl(options) {
	return options.apiUrl ?? process.env.API_URL ?? 'http://localhost:8080';
}

function toUpstreamUrl(sessionId, options) {
	const wsBase = resolveApiUrl(options).replace(/^http/, 'ws');
	return `${wsBase}/api/sessions/${sessionId}/ws`;
}

function connectUpstream(sessionId, accessToken, options) {
	return new WebSocket(toUpstreamUrl(sessionId, options), {
		headers: accessToken ? { authorization: `Bearer ${accessToken}` } : {}
	});
}

/**
 * Attaches the browser<->SvelteKit<->Rust WebSocket relay to an existing http.Server.
 * Safe to call with a null/undefined server (e.g. Vite in middleware mode with no httpServer).
 */
export function attachSessionStreamProxy(server, options = {}) {
	if (!server) return;
	const wss = new WebSocketServer({ noServer: true });

	server.on('upgrade', (request, socket, head) => {
		const url = new URL(request.url, 'http://internal');
		const match = url.pathname.match(SESSION_WS_PATH);
		if (!match) return; // not ours; let it fall through untouched

		const sessionId = match[1];
		const accessToken = parseCookie(request.headers.cookie ?? '').access_token;
		const upstream = connectUpstream(sessionId, accessToken, options);

		const cleanup = () => {
			upstream.off('open', onOpen);
			upstream.off('unexpected-response', onUnexpectedResponse);
			upstream.off('error', onError);
		};
		const onOpen = () => {
			cleanup();
			wss.handleUpgrade(request, socket, head, (browserWs) => {
				relay(browserWs, upstream, sessionId, accessToken, options);
			});
		};
		const onUnexpectedResponse = (_req, res) => {
			cleanup();
			const code = res.statusCode === 401 ? 4401 : res.statusCode === 404 ? 4404 : 1011;
			acceptThenClose(request, socket, head, wss, code);
		};
		const onError = () => {
			cleanup();
			acceptThenClose(request, socket, head, wss, 1011);
		};

		upstream.once('open', onOpen);
		upstream.once('unexpected-response', onUnexpectedResponse);
		upstream.once('error', onError);
	});
}

function acceptThenClose(request, socket, head, wss, code) {
	wss.handleUpgrade(request, socket, head, (browserWs) => {
		browserWs.close(code);
	});
}

/** Pipes an already-accepted browser socket to an already-open upstream, and owns the
 * upstream's reconnect-with-backoff for the rest of the browser socket's lifetime. */
function relay(browserWs, initialUpstream, sessionId, accessToken, options) {
	const initialRetryDelayMs = options.initialRetryDelayMs ?? DEFAULT_INITIAL_RETRY_DELAY_MS;
	const maxRetryDelayMs = options.maxRetryDelayMs ?? DEFAULT_MAX_RETRY_DELAY_MS;
	const maxTotalRetryMs = options.maxTotalRetryMs ?? DEFAULT_MAX_TOTAL_RETRY_MS;

	let upstream = initialUpstream;
	let finished = false;
	let retryDelay = initialRetryDelayMs;
	let retryDeadline = null;

	function finish(closeCode) {
		if (finished) return;
		finished = true;
		if (browserWs.readyState === WebSocket.OPEN || browserWs.readyState === WebSocket.CONNECTING) {
			browserWs.close(closeCode);
		}
		upstream?.close();
	}

	function wireUpstream(ws) {
		ws.on('message', (data) => {
			if (browserWs.readyState === WebSocket.OPEN) browserWs.send(data);
		});
		ws.on('error', () => {}); // 'close' always follows; that's what drives retry/give-up below
		ws.once('close', onUpstreamLost);
	}

	function onUpstreamLost() {
		if (finished) return;
		if (retryDeadline === null) retryDeadline = Date.now() + maxTotalRetryMs;
		if (Date.now() >= retryDeadline) {
			finish(1011);
			return;
		}
		const jitter = Math.random() * 250;
		setTimeout(attemptReconnect, retryDelay + jitter);
		retryDelay = Math.min(retryDelay * 2, maxRetryDelayMs);
	}

	function attemptReconnect() {
		if (finished) return;
		const next = connectUpstream(sessionId, accessToken, options);
		let settled = false;
		const settleOnce = (fn) => {
			if (settled) return;
			settled = true;
			fn();
		};

		next.once('open', () => {
			settleOnce(() => {
				if (finished) {
					next.close();
					return;
				}
				upstream = next;
				retryDeadline = null;
				retryDelay = initialRetryDelayMs;
				wireUpstream(next);
			});
		});
		next.once('unexpected-response', (_req, res) => {
			settleOnce(() => {
				if (res.statusCode === 401) finish(4401);
				else if (res.statusCode === 404) finish(4404);
				else onUpstreamLost();
			});
		});
		next.on('error', () => {});
		next.once('close', () => settleOnce(onUpstreamLost));
	}

	browserWs.on('message', (data) => {
		if (upstream.readyState === WebSocket.OPEN) upstream.send(data);
	});
	browserWs.once('close', () => finish(undefined));

	wireUpstream(upstream);
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cd frontend && npm run test:unit`
Expected: all 6 tests pass.

- [ ] **Step 7: Commit**

```bash
git add frontend/package.json frontend/package-lock.json frontend/vitest.config.ts frontend/ws-proxy/session-stream-proxy.js frontend/ws-proxy/session-stream-proxy.test.js
git commit -m "feat: add the browser<->SvelteKit<->Rust WebSocket relay module"
```

---

### Task 2: Wire the proxy into all three run modes

**Files:**
- Create: `frontend/server.js`
- Create: `frontend/vite-plugins/dev-ws-proxy.ts`
- Modify: `frontend/vite.config.ts`

**Interfaces:**
- Consumes: `attachSessionStreamProxy(server, options?)` from Task 1 (`frontend/ws-proxy/session-stream-proxy.js`), called with no `options` here (production defaults + `process.env.API_URL`).
- Produces: a working `/chat/:sessionId/ws` upgrade path in all three entrypoints — Task 4's e2e tests exercise this through `vite preview`; Task 3's browser code connects to it.

No new automated test in this task (the proxy logic itself is already covered by Task 1; this task is pure wiring). Verified by a build check plus a real boot-and-request smoke check against `server.js`, since that entrypoint has no other test coverage anywhere in this plan.

- [ ] **Step 1: Create the production entrypoint**

Create `frontend/server.js`:

```js
import { createServer } from 'node:http';
import { handler } from './build/handler.js';
import { attachSessionStreamProxy } from './ws-proxy/session-stream-proxy.js';

const port = process.env.PORT ?? 3000;
const host = process.env.HOST ?? '0.0.0.0';

const server = createServer(handler);
attachSessionStreamProxy(server);

server.listen(port, host, () => {
	console.log(`listening on http://${host}:${port}`);
});
```

- [ ] **Step 2: Create the Vite dev/preview plugin**

Create `frontend/vite-plugins/dev-ws-proxy.ts`:

```ts
import type { Plugin } from 'vite';
import { attachSessionStreamProxy } from '../ws-proxy/session-stream-proxy.js';

/**
 * Wires the same relay used in production (server.js) into `vite dev` and `vite preview` — the
 * latter is what the Playwright e2e harness boots via `npm run build && npm run preview`,
 * since SvelteKit's own Vite plugin serves the SSR build through `vite preview` via
 * `configurePreviewServer`, independently of adapter-node's server.js.
 */
export function devWsProxy(): Plugin {
	return {
		name: 'dev-ws-proxy',
		configureServer(server) {
			attachSessionStreamProxy(server.httpServer);
		},
		configurePreviewServer(server) {
			attachSessionStreamProxy(server.httpServer);
		}
	};
}
```

- [ ] **Step 3: Register the plugin**

Edit `frontend/vite.config.ts`:

```ts
import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { devWsProxy } from './vite-plugins/dev-ws-proxy';

export default defineConfig({
	plugins: [
		tailwindcss(),
		devWsProxy(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) => filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
			// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
			// See https://svelte.dev/docs/kit/adapters for more information about adapters.
			adapter: adapter()
		})
	]
});
```

- [ ] **Step 4: Verify the build succeeds**

Run: `cd frontend && npm run build`
Expected: builds cleanly, `build/handler.js` produced (as it already is today).

- [ ] **Step 5: Smoke-test the production entrypoint**

Run (adjust if `API_URL`/backend isn't reachable in this environment — the smoke check only needs to prove `server.js` boots and serves ordinary HTTP through `handler`, not that the WS path itself succeeds end-to-end, which Task 4 covers):

```bash
cd frontend
PORT=3100 node server.js &
SERVER_PID=$!
sleep 1
curl -s -o /dev/null -w '%{http_code}\n' http://localhost:3100/login
kill $SERVER_PID
```

Expected: prints `200` (or whatever the `/login` route's real status is today — confirm it matches what `npm run preview` already returns for the same path, proving `server.js` serves the app identically to the adapter's own default entrypoint), and the process starts/stops cleanly with no thrown errors in between.

- [ ] **Step 6: Commit**

```bash
git add frontend/server.js frontend/vite-plugins/dev-ws-proxy.ts frontend/vite.config.ts
git commit -m "feat: wire the WebSocket relay into dev, preview, and production entrypoints"
```

---

### Task 3: Fix the chat page's send flow and add live rendering

**Files:**
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts`
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`

**Interfaces:**
- Consumes: `/chat/{sessionId}/ws` from Task 2 (browser-side `new WebSocket(...)` call); `MessageItem` type (`frontend/src/lib/types.ts`, unchanged); `page.params.sessionId` from `$app/state`; `invalidateAll` from `$app/navigation`.
- Produces: nothing further downstream — Task 4's e2e tests exercise this page directly.

No new automated test in this task — Svelte component/page behavior in this codebase is proven through Playwright (Task 4), not a component-test framework (none exists here). Verified by `npm run check` passing cleanly.

- [ ] **Step 1: Fix the send action**

Edit `frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts` — replace the whole file:

```ts
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

		if (response.status === 404) {
			throw redirect(303, '/');
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to send message.' });
		}

		const { user_message } = (await response.json()) as { user_message: MessageItem };

		return { user_message };
	},
};
```

(Dropped: the `assistant_message` destructure and the `response.status === 502` / `turnFailed` branch — `POST /messages` is ingest-only and always either 202s or fails outright before any turn runs; a turn failure is now only observable asynchronously, over the WebSocket, per Task 1/2.)

- [ ] **Step 2: Add the WebSocket client and live-reply UI**

Edit `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte` — replace the whole file:

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let pendingReply = $state(false);
	let turnError = $state(false);
	let connectionLost = $state(false);

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;

	onMount(() => {
		let socket: WebSocket | undefined;
		let retryDelay = INITIAL_RETRY_DELAY_MS;
		let retryTimeout: ReturnType<typeof setTimeout> | undefined;
		let intentionallyClosed = false;
		let hasConnectedBefore = false;

		function connect() {
			socket = new WebSocket(`/chat/${page.params.sessionId}/ws`);

			socket.addEventListener('open', () => {
				retryDelay = INITIAL_RETRY_DELAY_MS;
				connectionLost = false;
				if (hasConnectedBefore) {
					// Reconnected after a drop; neither leg replays missed events, so re-fetch to
					// reconcile anything that happened while disconnected.
					invalidateAll();
				}
				hasConnectedBefore = true;
			});

			socket.addEventListener('message', (event) => {
				let envelope: { kind: string };
				try {
					envelope = JSON.parse(event.data);
				} catch {
					return;
				}
				if (envelope.kind === 'Delta') {
					pendingReply = true;
				} else if (envelope.kind === 'TurnCompleted') {
					pendingReply = false;
					turnError = false;
					invalidateAll();
				} else if (envelope.kind === 'TurnFailed') {
					pendingReply = false;
					turnError = true;
				}
			});

			socket.addEventListener('close', (event) => {
				if (intentionallyClosed) return;
				if (TERMINAL_CLOSE_CODES.has(event.code)) {
					connectionLost = true;
					return;
				}
				const jitter = Math.random() * 250;
				retryTimeout = setTimeout(connect, retryDelay + jitter);
				retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
			});
		}

		connect();

		return () => {
			intentionallyClosed = true;
			clearTimeout(retryTimeout);
			socket?.close();
		};
	});
</script>

<div class="flex h-full flex-col">
	<div class="flex-1 space-y-4 overflow-y-auto px-6 py-6">
		{#each data.messages as message (message.id)}
			<MessageBubble {message} />
		{/each}
		{#if pendingReply}
			<div class="flex justify-start">
				<div class="max-w-md rounded-2xl bg-neutral-100 px-4 py-2 text-neutral-400">Typing…</div>
			</div>
		{/if}
		{#if turnError}
			<p class="text-center text-sm text-red-600">Something went wrong — try sending again.</p>
		{/if}
		{#if form?.error}
			<p class="text-center text-sm text-red-600">{form.error}</p>
		{/if}
		{#if connectionLost}
			<p class="text-center text-sm text-red-600">Couldn't connect to this chat — try reloading the page.</p>
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

Note on the form's `use:enhance` callback: it's unchanged from before this plan — `update({ reset: true })` already re-runs `load` (and thus shows the just-sent message) as SvelteKit's default enhance behavior; no explicit extra `invalidateAll()` call is needed on the submit path itself, only from the WebSocket handlers above.

- [ ] **Step 3: Verify types check cleanly**

Run: `cd frontend && npm run check`
Expected: no new errors (pre-existing errors, if any, are out of scope for this task — only confirm this change didn't introduce new ones).

- [ ] **Step 4: Commit**

```bash
git add "frontend/src/routes/(app)/chat/[sessionId]/+page.server.ts" "frontend/src/routes/(app)/chat/[sessionId]/+page.svelte"
git commit -m "fix: repair the chat send flow and render live replies over the WebSocket relay"
```

---

### Task 4: End-to-end proof in a real browser

**Files:**
- Modify: `frontend/e2e/conversation.e2e.ts`

**Interfaces:**
- Consumes: everything from Tasks 1-3, exercised through a real browser against `vite preview` (via Playwright's existing `webServer` config) and the real backend + worker + EMQX (manually started — see the File Structure section's operational note above).
- Produces: nothing further downstream — this is the plan's last task.

**Before running this task's tests:** start `cargo run --bin worker` (from `backend/`) and confirm EMQX/Postgres are up (`docker compose up -d` in `backend/`), in addition to whatever Playwright's own `webServer` entries start automatically. These tests will hang until they hit Playwright's own timeout if the worker isn't running, since nothing will ever process the queued turn.

- [ ] **Step 1: Update the existing tests for the async reply path**

Edit `frontend/e2e/conversation.e2e.ts` — replace the whole file:

```ts
import { expect, test } from '@playwright/test';

// These tests exercise the live WebSocket-relayed reply path end to end. They require
// `cargo run --bin worker` (backend/src/bin/worker.rs) running alongside the backend server
// that playwright.config.ts's webServer entry starts, plus EMQX (backend/docker-compose.yml).
// The worker is not auto-started by Playwright — see
// docs/superpowers/plans/2026-08-17-frontend-realtime-rendering-implementation.md (File
// Structure section) for why. Start it manually before running this file.

async function registerAndStartChat(page: import('@playwright/test').Page): Promise<string> {
	const email = `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await page.getByRole('button', { name: 'New Chat', exact: true }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]+/);

	return email;
}

test('sending a message shows a typing indicator, then the fake assistant reply', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('hello there');
	await page.getByRole('button', { name: 'Send' }).click();

	await expect(page.getByRole('main').getByText('hello there')).toBeVisible();
	await expect(page.getByRole('main').getByText('Typing…')).toBeVisible();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();
	await expect(page.getByRole('main').getByText('Typing…')).toBeHidden();
});

test('a session history persists across a reload', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('remember this');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();

	await page.reload();
	await expect(page.getByRole('main').getByText('remember this')).toBeVisible();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();
});

test('visiting a nonexistent session redirects to the empty state', async ({ page }) => {
	await registerAndStartChat(page);

	await page.goto('/chat/00000000-0000-0000-0000-000000000000');
	await expect(page).toHaveURL('/');
});

test('a turn failure shows an inline error without losing the sent text', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('__SIMULATE_TURN_FAILURE__');
	await page.getByRole('button', { name: 'Send' }).click();

	await expect(page.getByRole('main').getByText('__SIMULATE_TURN_FAILURE__')).toBeVisible();
	await expect(page.getByText(/something went wrong/i)).toBeVisible();
});

test('a second turn on the same page still renders live, without reconnecting', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('first message');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();

	await page.getByPlaceholder('Ask me anything...').fill('second message');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.getByRole('main').getByText('second message')).toBeVisible();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toHaveCount(2);
});
```

(The `__SIMULATE_TURN_FAILURE__` sentinel is `backend/src/llm/fake.rs`'s existing failure-injection hook — its own doc comment already anticipates this exact use: "a turn failure now happens asynchronously in the worker... this no longer produces an observable HTTP-level failure," meaning the WebSocket relay this plan builds is what makes it observable again, this time as `TurnFailed`.)

- [ ] **Step 2: Run the e2e suite**

Ensure preconditions first:

```bash
cd backend && docker compose up -d
export DATABASE_URL="postgres://nomi:nomi@localhost:5432/nomi_dev"
export SETTINGS_ENCRYPTION_KEY="$(printf '0%.0s' {1..64})"
export LLM_PROVIDER=fake
export EMBEDDING_PROVIDER=fake
sqlx database create && sqlx migrate run
cargo run --bin worker &
WORKER_PID=$!
```

Then run: `cd frontend && npm run test:e2e`
Expected: all 5 tests in `conversation.e2e.ts` pass (plus whatever else is in the existing e2e suite, unaffected by this change).

Afterward: `kill $WORKER_PID`

- [ ] **Step 3: Commit**

```bash
git add frontend/e2e/conversation.e2e.ts
git commit -m "test: prove live reply rendering and turn-failure handling end to end"
```

---

## Out of Scope (unchanged from the design spec)

- Fixing the fake LLM provider to support failure injection — already supported (`__SIMULATE_TURN_FAILURE__`), discovered during planning; no backend change needed.
- Any bidirectional use of the browser↔SvelteKit WebSocket — outgoing messages still go through the existing POST form.
- Multi-tab/multi-device coordination beyond what falls out naturally from the connection being session-scoped.
- Auto-starting `cargo run --bin worker` from `playwright.config.ts` — no reliable readiness signal exists for it; documented as a manual precondition instead.
