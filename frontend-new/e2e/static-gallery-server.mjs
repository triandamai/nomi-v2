// Serves the static gallery (build-gallery/, from `npm run build:gallery`) under a nested path,
// the way a share host puts it somewhere other than the root. /health answers once the build is
// there, so Playwright waits for it.
import { createServer } from 'node:http';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../build-gallery/', import.meta.url));
const prefix = '/shared/nomi-ds/';
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.woff2': 'font/woff2', '.svg': 'image/svg+xml', '.png': 'image/png', '.jpg': 'image/jpeg', '.webp': 'image/webp' };

createServer((request, response) => {
	const path = decodeURIComponent(new URL(request.url ?? '/', 'http://localhost').pathname);
	if (path === '/health') {
		const ready = existsSync(join(root, 'design-system/nomi.html'));
		response.writeHead(ready ? 200 : 503).end(ready ? 'ok' : 'building');
		return;
	}
	const file = normalize(join(root, path.startsWith(prefix) ? path.slice(prefix.length) : '\0'));
	if (!file.startsWith(root) || !existsSync(file) || !statSync(file).isFile()) {
		response.writeHead(404).end('not found');
		return;
	}
	response.writeHead(200, { 'content-type': types[extname(file)] ?? 'application/octet-stream' }).end(readFileSync(file));
}).listen(4181, '127.0.0.1');
