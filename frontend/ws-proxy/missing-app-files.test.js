import { describe, it, expect, afterEach } from 'vitest';
import { createServer } from 'node:http';
import { noStoreMissingAppFiles } from './missing-app-files.js';

/** A server answering like adapter-node: headers set one by one, then writeHead(status). */
async function startServer(status, cacheControl) {
	const server = createServer((req, res) => {
		noStoreMissingAppFiles(req, res);
		res.setHeader('cache-control', cacheControl);
		res.writeHead(status).end('body');
	});
	await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
	return server;
}

let server;
afterEach(() => server?.close());

async function cacheControlOf(path, status, cacheControl) {
	server = await startServer(status, cacheControl);
	const response = await fetch(`http://127.0.0.1:${server.address().port}${path}`);
	return response.headers.get('cache-control');
}

describe('missing app files', () => {
	it('are never stored: a 404 mid-deploy must not outlive the deploy', async () => {
		expect(await cacheControlOf('/_app/immutable/nodes/29.abc.js', 404, 'public, max-age=0, must-revalidate')).toBe('no-store');
	});

	it('leave app files that exist cached as they were', async () => {
		expect(await cacheControlOf('/_app/immutable/nodes/29.abc.js', 200, 'public, max-age=31536000, immutable')).toBe('public, max-age=31536000, immutable');
	});

	it('leave a missing page alone', async () => {
		expect(await cacheControlOf('/no-such-page', 404, 'no-cache')).toBe('no-cache');
	});
});
