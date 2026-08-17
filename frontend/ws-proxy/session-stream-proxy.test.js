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
});
