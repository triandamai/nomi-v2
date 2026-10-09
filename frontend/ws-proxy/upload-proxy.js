import { request as httpRequest } from 'node:http';
import { request as httpsRequest } from 'node:https';
import { readCookie, resolveClientVersion } from './ws-proxy.js';

// Chat attachments (up to 50 MB) stream from the browser straight to the backend here, before
// SvelteKit's handler, whose body limit is far smaller. The same contract as
// src/routes/(app)/attachments/+server.ts (which serves `vite dev`): POST /attachments with the
// raw file as the body; a 401 goes back so the browser can refresh its session and retry.

const PASSED_HEADERS = ['content-type', 'content-length', 'x-file-name', 'x-attachment-kind', 'x-transcript'];

/**
 * @param {import('node:http').IncomingMessage} req
 */
function sameOrigin(req) {
	const origin = req.headers.origin;
	if (!origin) return true;
	try {
		return new URL(origin).host === req.headers.host;
	} catch {
		return false;
	}
}

/**
 * Handles `POST /attachments`; returns false for every other request.
 * @param {import('node:http').IncomingMessage} req
 * @param {import('node:http').ServerResponse} res
 * @param {{ apiUrl?: string, clientVersion?: string }} [options]
 */
export function handleUpload(req, res, options = {}) {
	const path = (req.url ?? '').split('?')[0];
	if (req.method !== 'POST' || path !== '/attachments') return false;

	// Cookies ride along on cross-site requests; an upload must come from Nomi's own pages.
	if (!sameOrigin(req)) {
		res.writeHead(403).end();
		return true;
	}
	const accessToken = readCookie(req.headers.cookie, 'access_token');
	if (!accessToken) {
		res.writeHead(401).end();
		return true;
	}

	const apiUrl = options.apiUrl ?? process.env.API_URL ?? 'http://localhost:8080';
	const target = new URL('/api/attachments', apiUrl);
	/** @type {Record<string, string>} */
	const headers = { authorization: `Bearer ${accessToken}`, 'x-client-version': resolveClientVersion(options) };
	for (const name of PASSED_HEADERS) {
		const value = req.headers[name];
		if (typeof value === 'string') headers[name] = value;
	}
	if (!headers['content-length']) headers['transfer-encoding'] = 'chunked';

	const send = target.protocol === 'https:' ? httpsRequest : httpRequest;
	const upstream = send(target, { method: 'POST', headers }, (response) => {
		res.writeHead(response.statusCode ?? 502, { 'content-type': response.headers['content-type'] ?? 'application/json' });
		response.pipe(res);
	});
	upstream.on('error', () => {
		if (!res.headersSent) res.writeHead(502);
		res.end();
	});
	// The browser gave up (or the backend refused early): stop sending.
	req.on('aborted', () => upstream.destroy());
	req.pipe(upstream);
	return true;
}
