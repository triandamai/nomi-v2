import { createServer } from 'node:http';
import { handler } from './build/handler.js';
import { attachWsProxy } from './ws-proxy/ws-proxy.js';

const port = process.env.PORT ?? 3000;
const host = process.env.HOST ?? '0.0.0.0';

const server = createServer(handler);
attachWsProxy(server);

server.listen(port, host, () => {
	console.log(`listening on http://${host}:${port}`);
});
