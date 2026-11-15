import { createServer } from 'node:http';
import { handler } from './build/handler.js';
import { attachWsProxy } from './ws-proxy/ws-proxy.js';
import { handleUpload } from './ws-proxy/upload-proxy.js';
import { handleSiteReport } from './ws-proxy/site-report-proxy.js';

const port = process.env.PORT ?? 3000;
const host = process.env.HOST ?? '0.0.0.0';

// Attachment uploads skip SvelteKit (and its request size limit) and stream to the backend; a
// published site's reports skip it too (it reads no body that comes without a content type).
const server = createServer((req, res) => {
	if (!handleUpload(req, res) && !handleSiteReport(req, res)) handler(req, res);
});
attachWsProxy(server);

server.listen(port, host, () => {
	console.log(`listening on http://${host}:${port}`);
});
