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
export function readCookie(cookieHeader, name) {
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

/**
 * Whether a request comes from Nomi's own pages. Cookies ride along on cross-site requests, so the
 * relay and the upload proxy only act on the session cookie for same-origin ones (or none, which
 * browsers always send on WebSocket handshakes and cross-site POSTs).
 * @param {import('node:http').IncomingMessage} req
 */
export function sameOrigin(req) {
	const origin = req.headers.origin;
	if (!origin) return true;
	try {
		return new URL(origin).host === req.headers.host;
	} catch {
		return false;
	}
}

/**
 * @typedef {Object} WsRoute
 * @property {RegExp} pattern
 * @property {(match: RegExpMatchArray) => string} upstreamPath
 */

/** @type {WsRoute[]} */
const ROUTES = [
	{
		pattern: /^\/chat\/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\/ws$/,
		upstreamPath: (match) => `/api/sessions/${match[1]}/ws`
	},
	{
		pattern: /^\/projects\/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\/ws$/,
		upstreamPath: (match) => `/api/projects/${match[1]}/ws`
	},
	{
		pattern: /^\/activity\/ws$/,
		upstreamPath: () => '/api/me/activity/ws'
	},
	{
		pattern: /^\/admin\/agents\/ws$/,
		upstreamPath: () => '/api/admin/agents/ws'
	}
];

/**
 * @param {string} pathname
 * @returns {(WsRoute & { match: RegExpMatchArray }) | undefined}
 */
function matchRoute(pathname) {
	for (const route of ROUTES) {
		const match = pathname.match(route.pattern);
		if (match) return { ...route, match };
	}
	return undefined;
}

const DEFAULT_INITIAL_RETRY_DELAY_MS = 1000;
const DEFAULT_MAX_RETRY_DELAY_MS = 30000;
const DEFAULT_MAX_TOTAL_RETRY_MS = 60000;

/**
 * @typedef {Object} ProxyOptions
 * @property {string} [apiUrl]
 * @property {number} [initialRetryDelayMs]
 * @property {number} [maxRetryDelayMs]
 * @property {number} [maxTotalRetryMs]
 * @property {string} [clientVersion] Sent upstream as X-Client-Version; defaults to APP_VERSION.
 */

/**
 * @param {ProxyOptions} options
 * @returns {string}
 */
function resolveApiUrl(options) {
	return options.apiUrl ?? process.env.API_URL ?? 'http://localhost:8080';
}

/**
 * @param {string} upstreamPath
 * @param {ProxyOptions} options
 * @returns {string}
 */
function toUpstreamUrl(upstreamPath, options) {
	const wsBase = resolveApiUrl(options).replace(/^http/, 'ws');
	return `${wsBase}${upstreamPath}`;
}

/**
 * @param {string} upstreamPath
 * @param {string | undefined} accessToken
 * @param {ProxyOptions} options
 * @returns {WebSocket}
 */
function connectUpstream(upstreamPath, accessToken, options) {
	/** @type {Record<string, string>} */
	const headers = { 'x-client-version': resolveClientVersion(options) };
	if (accessToken) headers.authorization = `Bearer ${accessToken}`;
	return new WebSocket(toUpstreamUrl(upstreamPath, options), { headers });
}

/**
 * @param {ProxyOptions} options
 * @returns {string}
 */
export function resolveClientVersion(options) {
	return options.clientVersion ?? process.env.APP_VERSION?.trim().replace(/^v(?=\d)/, '') ?? 'unknown';
}

/**
 * The person's tokens for one relayed connection. The access token is swapped for a fresh one
 * when the backend refuses it, so a connection outlives the access token (minutes) as long as the
 * refresh token (weeks) is good. The browser's cookies aren't touched: its next page load
 * refreshes them the same way.
 * @typedef {{ accessToken?: string, refreshToken?: string }} RelaySession
 */

/**
 * A new access token from the refresh token, or null when there's none or the API refuses it.
 * @param {string | undefined} refreshToken
 * @param {ProxyOptions} options
 * @returns {Promise<string | null>}
 */
async function refreshAccessToken(refreshToken, options) {
	if (!refreshToken) return null;
	try {
		const response = await fetch(new URL('/api/auth/refresh', resolveApiUrl(options)), {
			method: 'POST',
			headers: { 'content-type': 'application/json', 'x-client-version': resolveClientVersion(options) },
			body: JSON.stringify({ refresh_token: refreshToken }),
		});
		if (!response.ok) return null;
		const body = /** @type {{ access_token?: unknown }} */ (await response.json());
		return typeof body.access_token === 'string' ? body.access_token : null;
	} catch {
		return null;
	}
}

/**
 * Opens the upstream leg. A 401 gets one refresh and one more try. Calls back with the open
 * socket, or with the close code to give the browser: 4401 (signed out), 4404 (no such chat), or
 * null when the backend couldn't be reached (the caller decides whether to retry). The callback
 * runs inside the socket's 'open' event, not a later tick: the backend may send its first message
 * in the same packet as the handshake, and it must find the relay already listening.
 * @param {string} upstreamPath
 * @param {RelaySession} session
 * @param {ProxyOptions} options
 * @param {(opened: { ws: WebSocket, code?: undefined } | { ws?: undefined, code: number | null }) => void} done
 */
function openUpstream(upstreamPath, session, options, done) {
	{
		const resolve = done;
		/** @param {boolean} mayRefresh */
		const attempt = (mayRefresh) => {
			const ws = connectUpstream(upstreamPath, session.accessToken, options);
			let handled = false;
			ws.on('error', () => {}); // 'close' always follows
			ws.once('open', () => {
				if (handled) return;
				handled = true;
				resolve({ ws });
			});
			ws.once('unexpected-response', async (_request, response) => {
				if (handled) return;
				handled = true;
				if (response.statusCode === 401 && mayRefresh) {
					const token = await refreshAccessToken(session.refreshToken, options);
					if (token) {
						session.accessToken = token;
						attempt(false);
						return;
					}
				}
				resolve({ code: response.statusCode === 401 ? 4401 : response.statusCode === 404 ? 4404 : null });
			});
			ws.once('close', () => {
				if (handled) return;
				handled = true;
				resolve({ code: null });
			});
		};
		attempt(true);
	}
}

/**
 * Attaches the browser<->SvelteKit<->Rust WebSocket relay to an existing http.Server.
 * Safe to call with a null/undefined server (e.g. Vite in middleware mode with no httpServer).
 * @param {import('node:http').Server | import('node:http2').Http2SecureServer | null | undefined} server
 * @param {ProxyOptions} [options]
 */
export function attachWsProxy(server, options = {}) {
	if (!server) return;
	const wss = new WebSocketServer({ noServer: true });

	server.on('upgrade', (request, socket, head) => {
		const url = new URL(request.url ?? '', 'http://internal');
		const route = matchRoute(url.pathname);
		if (!route) return; // not ours; let it fall through untouched

		if (!sameOrigin(request)) {
			socket.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\nContent-Length: 0\r\n\r\n');
			return;
		}

		const upstreamPath = route.upstreamPath(route.match);
		/** @type {RelaySession} */
		const session = {
			accessToken: readCookie(request.headers.cookie, 'access_token'),
			refreshToken: readCookie(request.headers.cookie, 'refresh_token'),
		};
		openUpstream(upstreamPath, session, options, (opened) => {
			if (opened.ws) {
				const upstream = opened.ws;
				wss.handleUpgrade(request, socket, head, (browserWs) => relay(browserWs, upstream, upstreamPath, session, options));
			} else {
				acceptThenClose(request, socket, head, wss, opened.code ?? 1011);
			}
		});
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
 * @param {string} upstreamPath
 * @param {RelaySession} session
 * @param {ProxyOptions} options
 */
function relay(browserWs, initialUpstream, upstreamPath, session, options) {
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
		ws.on('message', (data, isBinary) => {
			if (browserWs.readyState === WebSocket.OPEN) browserWs.send(data, { binary: isBinary });
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
		openUpstream(upstreamPath, session, options, (opened) => {
			if (!opened.ws) {
				if (opened.code) finish(opened.code);
				else onUpstreamLost();
				return;
			}
			if (finished) {
				opened.ws.close();
				return;
			}
			upstream = opened.ws;
			retryDeadline = null;
			retryDelay = initialRetryDelayMs;
			wireUpstream(opened.ws);
		});
	}

	browserWs.on('message', (data, isBinary) => {
		if (upstream.readyState === WebSocket.OPEN) upstream.send(data, { binary: isBinary });
	});
	browserWs.on('error', () => {}); // prevent unhandled-error crash; actual disconnect handling happens via 'close'
	browserWs.once('close', () => finish(undefined));

	wireUpstream(upstream);
}
