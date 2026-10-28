import { describe, it, expect, afterEach } from 'vitest';
import { createServer } from 'node:http';
import { WebSocketServer, WebSocket } from 'ws';
import { attachWsProxy } from './ws-proxy.js';

const SESSION_ID = '11111111-1111-1111-1111-111111111111';

/** Starts a bare http.Server with the proxy attached, listening on an ephemeral port. */
async function startProxyServer(options) {
	const server = createServer((_req, res) => {
		res.statusCode = 404;
		res.end();
	});
	attachWsProxy(server, options);
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

/**
 * A fake backend that checks the access token on every WebSocket handshake and can refresh it:
 * `POST /api/auth/refresh` answers with `refreshTo` (or 401 when it's null).
 */
async function startAuthUpstream({ accepts, refreshTo }) {
	const refreshes = [];
	const server = createServer((req, res) => {
		if (req.method === 'POST' && req.url === '/api/auth/refresh') {
			let body = '';
			req.on('data', (chunk) => (body += chunk));
			req.on('end', () => {
				refreshes.push(JSON.parse(body));
				if (!refreshTo) return res.writeHead(401).end();
				res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify({ access_token: refreshTo }));
			});
			return;
		}
		res.writeHead(404).end();
	});
	const wss = new WebSocketServer({ noServer: true });
	server.on('upgrade', (request, socket, head) => {
		const token = (request.headers.authorization ?? '').replace(/^Bearer /, '');
		if (!accepts(token)) {
			socket.end('HTTP/1.1 401 Unauthorized\r\n\r\n');
			return;
		}
		wss.handleUpgrade(request, socket, head, (ws) => wss.emit('connection', ws, request));
	});
	await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
	return { server, wss, refreshes, url: `http://127.0.0.1:${server.address().port}` };
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

	it('tells the upstream which frontend version is connecting', async () => {
		const upstream = await startFakeUpstream();
		cleanups.push(() => upstream.server.close());
		const connected = new Promise((resolve) => upstream.wss.on('connection', (_ws, request) => resolve(request.headers)));

		const proxy = await startProxyServer({ apiUrl: upstream.url, clientVersion: '1.2.3' });
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port);
		cleanups.push(() => browser.close());
		const headers = await connected;
		// Let the browser leg finish its handshake too, so closing it in cleanup is clean.
		await waitFor(browser, 'open');
		expect(headers['x-client-version']).toBe('1.2.3');
		expect(headers.authorization).toBe('Bearer test-token');
	});

	it("relays a project's live updates from the project's own upstream path", async () => {
		const upstream = await startFakeUpstream();
		cleanups.push(() => upstream.server.close());
		const path = new Promise((resolve) => upstream.wss.on('connection', (ws, request) => {
			ws.send('changed');
			resolve(request.url);
		}));

		const proxy = await startProxyServer({ apiUrl: upstream.url });
		cleanups.push(() => proxy.server.close());

		const browser = new WebSocket(`ws://127.0.0.1:${proxy.port}/projects/${SESSION_ID}/ws`, { headers: { cookie: 'access_token=test-token' } });
		cleanups.push(() => browser.close());
		const [message] = await waitFor(browser, 'message');
		expect(await path).toBe(`/api/projects/${SESSION_ID}/ws`);
		expect(message.toString()).toBe('changed');
	});

	it("relays the person's own activity feed from the user's upstream path", async () => {
		const upstream = await startFakeUpstream();
		cleanups.push(() => upstream.server.close());
		const path = new Promise((resolve) => upstream.wss.on('connection', (ws, request) => {
			ws.send('finished');
			resolve(request.url);
		}));

		const proxy = await startProxyServer({ apiUrl: upstream.url });
		cleanups.push(() => proxy.server.close());

		const browser = new WebSocket(`ws://127.0.0.1:${proxy.port}/activity/ws`, { headers: { cookie: 'access_token=test-token' } });
		cleanups.push(() => browser.close());
		const [message] = await waitFor(browser, 'message');
		expect(await path).toBe('/api/me/activity/ws');
		expect(message.toString()).toBe('finished');
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
		first.wss.clients.forEach((ws) => ws.terminate());
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

		first.wss.clients.forEach((ws) => ws.terminate());
		await new Promise((resolve) => first.server.close(resolve));
		// Nothing is listening on first.port again from here on.

		const [code] = await waitFor(browser, 'close');
		expect(code).toBe(1011);
	});
	it('refuses a connection opened from another site', async () => {
		const upstream = await startFakeUpstream();
		cleanups.push(() => upstream.server.close());
		const proxy = await startProxyServer({ apiUrl: upstream.url });
		cleanups.push(() => proxy.server.close());

		// The session cookie rides along on cross-site requests; the relay must not use it for them.
		const browser = new WebSocket(`ws://127.0.0.1:${proxy.port}/chat/${SESSION_ID}/ws`, {
			headers: { cookie: 'access_token=test-token', origin: 'https://evil.example' },
		});
		// Refused during the handshake: it never opens, so closing it later reports an error.
		browser.on('error', () => {});
		cleanups.push(() => browser.terminate());
		const [, response] = await waitFor(browser, 'unexpected-response');
		expect(response.statusCode).toBe(403);
		expect(upstream.wss.clients.size).toBe(0);
	});

	it('accepts a connection from its own origin', async () => {
		const upstream = await startFakeUpstream();
		cleanups.push(() => upstream.server.close());
		const proxy = await startProxyServer({ apiUrl: upstream.url });
		cleanups.push(() => proxy.server.close());

		const browser = new WebSocket(`ws://127.0.0.1:${proxy.port}/chat/${SESSION_ID}/ws`, {
			headers: { cookie: 'access_token=test-token', origin: `http://127.0.0.1:${proxy.port}` },
		});
		cleanups.push(() => browser.close());
		await waitFor(browser, 'open');
		expect(browser.readyState).toBe(WebSocket.OPEN);
	});

	it('signs back in with the refresh token when the access token has expired', async () => {
		const upstream = await startAuthUpstream({ accepts: (token) => token === 'fresh', refreshTo: 'fresh' });
		cleanups.push(() => upstream.server.close());
		const connected = new Promise((resolve) => upstream.wss.on('connection', (_ws, request) => resolve(request.headers.authorization)));
		const proxy = await startProxyServer({ apiUrl: upstream.url });
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port, SESSION_ID, 'access_token=expired; refresh_token=r1');
		cleanups.push(() => browser.close());
		await waitFor(browser, 'open');
		expect(await connected).toBe('Bearer fresh');
		expect(upstream.refreshes).toEqual([{ refresh_token: 'r1' }]);
	});

	it('refreshes the token when a reconnect is refused, so live updates outlast the access token', async () => {
		let valid = 'first';
		const upstream = await startAuthUpstream({ accepts: (token) => token === valid, refreshTo: 'second' });
		cleanups.push(() => upstream.server.close());
		upstream.wss.on('connection', (ws, request) => ws.on('message', () => ws.send(request.headers.authorization)));

		const proxy = await startProxyServer({ apiUrl: upstream.url, initialRetryDelayMs: 20, maxRetryDelayMs: 50, maxTotalRetryMs: 2000 });
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port, SESSION_ID, 'access_token=first; refresh_token=r1');
		cleanups.push(() => browser.close());
		await waitFor(browser, 'open');

		// The access token expires while the connection is up, then the backend drops it.
		valid = 'second';
		upstream.wss.clients.forEach((ws) => ws.terminate());
		await new Promise((resolve) => setTimeout(resolve, 300));

		browser.send('who');
		const [reply] = await waitFor(browser, 'message');
		expect(reply.toString()).toBe('Bearer second');
		expect(browser.readyState).toBe(WebSocket.OPEN);
	});

	it('closes with 4401 when the refresh token is no longer accepted either', async () => {
		const upstream = await startAuthUpstream({ accepts: () => false, refreshTo: null });
		cleanups.push(() => upstream.server.close());
		const proxy = await startProxyServer({ apiUrl: upstream.url });
		cleanups.push(() => proxy.server.close());

		const browser = connectBrowserClient(proxy.port, SESSION_ID, 'access_token=expired; refresh_token=gone');
		cleanups.push(() => browser.close());
		const [code] = await waitFor(browser, 'close');
		expect(code).toBe(4401);
		expect(upstream.refreshes).toHaveLength(1);
	});
});
