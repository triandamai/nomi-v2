// `npm run dev:demo`: Home with demo data, no backend and no sign-in.
//
//   http://localhost:5175  →  a small gate that signs every request in as the demo account
//                          →  `vite dev` on 5176, whose API calls go to the demo API on 4191.
//
// For looking at the UI only. The gate forwards WebSocket upgrades too, so hot reload works.
// Options (environment): DEMO_THEME=light|dark|system, DEMO_LANGUAGE=en|id.
import { spawn } from 'node:child_process';
import http from 'node:http';
import net from 'node:net';
import { startDemoApi } from './api.mjs';

const GATE = 5175;
const APP = 5176;
// Not 4190: fetch refuses that port (it's on the browsers' blocked list).
const API = 4191;

const signedIn = (cookie = '') => (/(^|;\s*)access_token=/.test(cookie) ? cookie : [cookie, 'access_token=demo'].filter(Boolean).join('; '));

await startDemoApi(API);

// --host: on a Mac `localhost` is ::1 (IPv6), where the gate, on 127.0.0.1, wouldn't find it.
const vite = spawn('npx', ['vite', 'dev', '--host', '127.0.0.1', '--port', String(APP), '--strictPort'], {
	stdio: ['ignore', 'pipe', 'inherit'],
	env: { ...process.env, API_URL: `http://127.0.0.1:${API}` },
});
vite.stdout.on('data', () => {}); // vite's own banner names 5176; the address to open is the gate's
const stop = () => {
	vite.kill();
	process.exit();
};
process.on('SIGINT', stop);
process.on('SIGTERM', stop);
vite.on('exit', (code) => process.exit(code ?? 0));

const gate = http.createServer((req, res) => {
	const upstream = http.request({ host: '127.0.0.1', port: APP, method: req.method, path: req.url, headers: { ...req.headers, cookie: signedIn(req.headers.cookie) } }, (reply) => {
		res.writeHead(reply.statusCode ?? 502, reply.headers);
		reply.pipe(res);
	});
	upstream.on('error', () => res.writeHead(502, { 'content-type': 'text/plain' }).end('The dev server is still starting. Reload in a moment.'));
	req.pipe(upstream);
});

// Hot reload and live updates: pass the upgrade through as raw bytes.
gate.on('upgrade', (req, socket, head) => {
	const upstream = net.connect(APP, '127.0.0.1', () => {
		const headers = { ...req.headers, cookie: signedIn(req.headers.cookie) };
		const lines = [`${req.method} ${req.url} HTTP/${req.httpVersion}`, ...Object.entries(headers).map(([key, value]) => `${key}: ${value}`)];
		upstream.write(lines.join('\r\n') + '\r\n\r\n');
		if (head?.length) upstream.write(head);
		socket.pipe(upstream).pipe(socket);
	});
	upstream.on('error', () => socket.destroy());
	socket.on('error', () => upstream.destroy());
});

gate.listen(GATE, () => console.log(`\n  Nomi demo (signed in as Budi Santoso, demo data): http://localhost:${GATE}\n`));
