// A stand-in for the backend API, for the e2e smoke tests: it accepts the access token
// `e2e-token`, renews `e2e-refresh` into it, and serves the person's activity feed.
import { createServer } from 'node:http';
import { WebSocketServer } from 'ws';

const port = Number(process.env.STUB_API_PORT ?? 4180);

const server = createServer((req, res) => {
	if (req.method === 'POST' && req.url === '/api/auth/refresh') {
		let body = '';
		req.on('data', (chunk) => (body += chunk));
		req.on('end', () => {
			const { refresh_token } = JSON.parse(body || '{}');
			if (refresh_token !== 'e2e-refresh') return res.writeHead(401).end();
			res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify({ access_token: 'e2e-token' }));
		});
		return;
	}
	if (req.url === '/health') return res.writeHead(200).end('ok');
	res.writeHead(404).end();
});

const wss = new WebSocketServer({ noServer: true });
server.on('upgrade', (request, socket, head) => {
	if (request.url !== '/api/me/activity/ws' || request.headers.authorization !== 'Bearer e2e-token') {
		socket.end('HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n');
		return;
	}
	wss.handleUpgrade(request, socket, head, (ws) => ws.send(JSON.stringify({ kind: 'Hello' })));
});

server.listen(port, '127.0.0.1', () => console.log(`stub API on http://127.0.0.1:${port}`));
