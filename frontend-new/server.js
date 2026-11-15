import { createServer } from 'node:http';
import { attachWsProxy } from './ws-proxy/ws-proxy.js';
import { handleUpload } from './ws-proxy/upload-proxy.js';

// SvelteKit adapter-node defaults BODY_SIZE_LIMIT to 512K unless overridden before handler loads.
process.env.BODY_SIZE_LIMIT = process.env.BODY_SIZE_LIMIT ?? 'Infinity';
const { handler } = await import('./build/handler.js');

const port = process.env.PORT ?? 3000;
const host = process.env.HOST ?? '0.0.0.0';

// Attachment uploads skip SvelteKit (and its request size limit) and stream to the backend.
const server = createServer((req, res) => {
	if (!handleUpload(req, res)) handler(req, res);
});
attachWsProxy(server);

server.listen(port, host, () => {
	console.log(`listening on http://${host}:${port}`);
});
