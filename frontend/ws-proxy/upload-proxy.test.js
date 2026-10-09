import { describe, it, expect, afterEach } from 'vitest';
import { createServer } from 'node:http';
import { handleUpload } from './upload-proxy.js';

/** A fake backend that records the upload it receives. */
async function startFakeBackend(status = 201) {
	const received = [];
	const server = createServer((req, res) => {
		const chunks = [];
		req.on('data', (c) => chunks.push(c));
		req.on('end', () => {
			received.push({ url: req.url, headers: req.headers, body: Buffer.concat(chunks) });
			res.writeHead(status, { 'content-type': 'application/json' });
			res.end(JSON.stringify({ id: 'abc' }));
		});
	});
	await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
	return { server, received, url: `http://127.0.0.1:${server.address().port}` };
}

async function startProxy(apiUrl) {
	const server = createServer((req, res) => {
		if (!handleUpload(req, res, { apiUrl, clientVersion: '9.9.9' })) {
			res.writeHead(404).end('sveltekit');
		}
	});
	await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
	return { server, url: `http://127.0.0.1:${server.address().port}` };
}

describe('upload-proxy', () => {
	let cleanups = [];
	afterEach(() => {
		cleanups.forEach((fn) => fn());
		cleanups = [];
	});

	it('streams the file to the backend with the session and file headers', async () => {
		const backend = await startFakeBackend();
		const proxy = await startProxy(backend.url);
		cleanups.push(() => backend.server.close(), () => proxy.server.close());

		const body = Buffer.alloc(3 * 1024 * 1024, 7);
		const response = await fetch(`${proxy.url}/attachments`, {
			method: 'POST',
			headers: { cookie: 'access_token=tok', 'content-type': 'image/png', 'x-file-name': 'Struk%20Mei.png', 'x-attachment-kind': 'voice' },
			body,
		});
		expect(response.status).toBe(201);
		expect(await response.json()).toEqual({ id: 'abc' });
		const [upload] = backend.received;
		expect(upload.url).toBe('/api/attachments');
		expect(upload.headers.authorization).toBe('Bearer tok');
		expect(upload.headers['x-client-version']).toBe('9.9.9');
		expect(upload.headers['x-file-name']).toBe('Struk%20Mei.png');
		expect(upload.headers['content-type']).toBe('image/png');
		expect(upload.body.equals(body)).toBe(true);
	});

	it('refuses uploads without a session or from another site, and leaves other requests alone', async () => {
		const backend = await startFakeBackend();
		const proxy = await startProxy(backend.url);
		cleanups.push(() => backend.server.close(), () => proxy.server.close());

		expect((await fetch(`${proxy.url}/attachments`, { method: 'POST', body: 'x' })).status).toBe(401);
		const crossSite = await fetch(`${proxy.url}/attachments`, { method: 'POST', body: 'x', headers: { cookie: 'access_token=tok', origin: 'https://evil.example' } });
		expect(crossSite.status).toBe(403);
		expect(await (await fetch(`${proxy.url}/attachments/123/content`)).text()).toBe('sveltekit');
		expect(backend.received).toHaveLength(0);
	});

	it('passes the backend\'s refusal back', async () => {
		const backend = await startFakeBackend(413);
		const proxy = await startProxy(backend.url);
		cleanups.push(() => backend.server.close(), () => proxy.server.close());
		const response = await fetch(`${proxy.url}/attachments`, { method: 'POST', body: 'x', headers: { cookie: 'access_token=tok' } });
		expect(response.status).toBe(413);
	});
});
