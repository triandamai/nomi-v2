import { WebSocketServer, WebSocket } from 'ws';
import { parseCookie } from 'cookie';

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
