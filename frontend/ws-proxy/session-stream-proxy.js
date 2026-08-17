import { WebSocketServer, WebSocket } from 'ws';

// Not the 'cookie' npm package: a bare 'cookie' dependency here collides with
// @sveltejs/kit's own nested cookie@0.6.0 once Kit's SSR bundle externalizes its own
// `import ... from "cookie"` (Node's resolution from .svelte-kit/output/server/index.js
// can't see Kit's nested node_modules/@sveltejs/kit/node_modules/cookie once a
// conflicting top-level 'cookie' package exists). We only need one named cookie value
// out of a raw Cookie header, so a few lines beats fighting npm's hoisting.
/**
 * @param {string | undefined} cookieHeader
 * @param {string} name
 * @returns {string | undefined}
 */
function readCookie(cookieHeader, name) {
	if (!cookieHeader) return undefined;
	for (const pair of cookieHeader.split(';')) {
		const eq = pair.indexOf('=');
		if (eq === -1) continue;
		if (pair.slice(0, eq).trim() !== name) continue;
		const value = pair.slice(eq + 1).trim();
		try {
			return decodeURIComponent(value);
		} catch {
			return value;
		}
	}
	return undefined;
}

const SESSION_WS_PATH =
	/^\/chat\/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\/ws$/;

const DEFAULT_INITIAL_RETRY_DELAY_MS = 1000;
const DEFAULT_MAX_RETRY_DELAY_MS = 30000;
const DEFAULT_MAX_TOTAL_RETRY_MS = 60000;

/**
 * @typedef {Object} ProxyOptions
 * @property {string} [apiUrl]
 * @property {number} [initialRetryDelayMs]
 * @property {number} [maxRetryDelayMs]
 * @property {number} [maxTotalRetryMs]
 */

/**
 * @param {ProxyOptions} options
 * @returns {string}
 */
function resolveApiUrl(options) {
	return options.apiUrl ?? process.env.API_URL ?? 'http://localhost:8080';
}

/**
 * @param {string} sessionId
 * @param {ProxyOptions} options
 * @returns {string}
 */
function toUpstreamUrl(sessionId, options) {
	const wsBase = resolveApiUrl(options).replace(/^http/, 'ws');
	return `${wsBase}/api/sessions/${sessionId}/ws`;
}

/**
 * @param {string} sessionId
 * @param {string | undefined} accessToken
 * @param {ProxyOptions} options
 * @returns {WebSocket}
 */
function connectUpstream(sessionId, accessToken, options) {
	return new WebSocket(toUpstreamUrl(sessionId, options), {
		headers: accessToken ? { authorization: `Bearer ${accessToken}` } : {}
	});
}

/**
 * Attaches the browser<->SvelteKit<->Rust WebSocket relay to an existing http.Server.
 * Safe to call with a null/undefined server (e.g. Vite in middleware mode with no httpServer).
 * @param {import('node:http').Server | import('node:http2').Http2SecureServer | null | undefined} server
 * @param {ProxyOptions} [options]
 */
export function attachSessionStreamProxy(server, options = {}) {
	if (!server) return;
	const wss = new WebSocketServer({ noServer: true });

	server.on('upgrade', (request, socket, head) => {
		const url = new URL(request.url ?? '', 'http://internal');
		const match = url.pathname.match(SESSION_WS_PATH);
		if (!match) return; // not ours; let it fall through untouched

		const sessionId = match[1];
		const accessToken = readCookie(request.headers.cookie, 'access_token');
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
		/**
		 * @param {import('node:http').ClientRequest} _req
		 * @param {import('node:http').IncomingMessage} res
		 */
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

/**
 * @param {import('node:http').IncomingMessage} request
 * @param {import('node:stream').Duplex} socket
 * @param {Buffer} head
 * @param {WebSocketServer} wss
 * @param {number} code
 */
function acceptThenClose(request, socket, head, wss, code) {
	wss.handleUpgrade(request, socket, head, (browserWs) => {
		browserWs.on('error', () => {}); // prevent unhandled-error crash; actual disconnect handling happens via 'close'
		browserWs.close(code);
	});
}

/**
 * Pipes an already-accepted browser socket to an already-open upstream, and owns the
 * upstream's reconnect-with-backoff for the rest of the browser socket's lifetime.
 * @param {WebSocket} browserWs
 * @param {WebSocket} initialUpstream
 * @param {string} sessionId
 * @param {string | undefined} accessToken
 * @param {ProxyOptions} options
 */
function relay(browserWs, initialUpstream, sessionId, accessToken, options) {
	const initialRetryDelayMs = options.initialRetryDelayMs ?? DEFAULT_INITIAL_RETRY_DELAY_MS;
	const maxRetryDelayMs = options.maxRetryDelayMs ?? DEFAULT_MAX_RETRY_DELAY_MS;
	const maxTotalRetryMs = options.maxTotalRetryMs ?? DEFAULT_MAX_TOTAL_RETRY_MS;

	let upstream = initialUpstream;
	let finished = false;
	let retryDelay = initialRetryDelayMs;
	/** @type {number | null} */
	let retryDeadline = null;

	/** @param {number | undefined} closeCode */
	function finish(closeCode) {
		if (finished) return;
		finished = true;
		if (browserWs.readyState === WebSocket.OPEN || browserWs.readyState === WebSocket.CONNECTING) {
			browserWs.close(closeCode);
		}
		upstream?.close();
	}

	/** @param {WebSocket} ws */
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
		/** @param {() => void} fn */
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
	browserWs.on('error', () => {}); // prevent unhandled-error crash; actual disconnect handling happens via 'close'
	browserWs.once('close', () => finish(undefined));

	wireUpstream(upstream);
}
