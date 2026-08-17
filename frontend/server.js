import { createServer } from 'node:http';
import { handler } from './build/handler.js';
import { attachSessionStreamProxy } from './ws-proxy/session-stream-proxy.js';

const port = process.env.PORT ?? 3000;
const host = process.env.HOST ?? '0.0.0.0';

const server = createServer(handler);
attachSessionStreamProxy(server);

server.listen(port, host, () => {
	console.log(`listening on http://${host}:${port}`);
});
