import { describe, it, expect, afterEach } from 'vitest';
import { createServer } from 'node:http';
import { handleSiteReport } from './site-report-proxy.js';

/** A fake backend that records what reaches it. */
async function startFakeBackend() {
	const received = [];
	const server = createServer((req, res) => {
		const chunks = [];
		req.on('data', (chunk) => chunks.push(chunk));
		req.on('end', () => {
			received.push({ url: req.url, headers: req.headers, body: Buffer.concat(chunks).toString() });
			res.writeHead(204).end();
		});
	});
	await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
	return { server, received, url: `http://127.0.0.1:${server.address().port}` };
}

async function startProxy(apiUrl) {
	const server = createServer((req, res) => {
		if (!handleSiteReport(req, res, { apiUrl })) res.writeHead(404).end('sveltekit');
	});
	await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
	return { server, url: `http://127.0.0.1:${server.address().port}` };
}

const servers = [];
afterEach(() => {
	for (const server of servers.splice(0)) server.close();
});

describe('a published site’s report', () => {
	it('goes on to the backend as it came, without a content type (a beacon’s untyped Blob) and from another site', async () => {
		const backend = await startFakeBackend();
		const proxy = await startProxy(backend.url);
		servers.push(backend.server, proxy.server);
		const report = '{"k":"v","r":"https://www.instagram.com/","h":"warungkopi.id"}';
		const id = '0b6f1c2e-6a3b-4f0e-9d1a-2c3b4d5e6f70';
		const response = await fetch(`${proxy.url}/hooks/sites/${id}`, { method: 'POST', body: new Blob([report]), headers: { origin: 'https://warungkopi.id', cookie: 'access_token=secret' } });
		expect([response.status, response.headers.get('access-control-allow-origin')]).toEqual([204, '*']);
		await expect.poll(() => backend.received.length).toBe(1);
		expect(backend.received[0].url).toBe(`/api/hooks/sites/${id}`);
		expect(backend.received[0].body).toBe(report);
		expect(backend.received[0].headers.cookie).toBeUndefined();
		expect(backend.received[0].headers.origin).toBeUndefined();
	});

	it('drops a report too big to be one, and leaves every other request to the app', async () => {
		const backend = await startFakeBackend();
		const proxy = await startProxy(backend.url);
		servers.push(backend.server, proxy.server);
		const big = await fetch(`${proxy.url}/hooks/sites/0b6f1c2e-6a3b-4f0e-9d1a-2c3b4d5e6f70`, { method: 'POST', body: 'x'.repeat(5000) });
		expect(big.status).toBe(204);
		for (const [method, path] of [['GET', '/hooks/sites/0b6f1c2e-6a3b-4f0e-9d1a-2c3b4d5e6f70'], ['POST', '/hooks/sites/not-an-id'], ['POST', '/hooks/sites/0b6f1c2e-6a3b-4f0e-9d1a-2c3b4d5e6f70/more'], ['POST', '/hooks/shipyard']]) {
			const other = await fetch(`${proxy.url}${path}`, { method, ...(method === 'POST' && { body: '{}' }) });
			expect(await other.text()).toBe('sveltekit');
		}
		expect(backend.received).toEqual([]);
	});
});
