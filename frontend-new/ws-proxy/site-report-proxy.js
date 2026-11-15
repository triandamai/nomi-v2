import { request as httpRequest } from 'node:http';
import { request as httpsRequest } from 'node:https';

// A published site reports each visit's referrer and each form sent to APP_URL/hooks/sites/:id
// (the script a publish puts in its pages: `navigator.sendBeacon` with an untyped Blob). It's
// handled here, before SvelteKit, whose Node handler reads no body without a content type, so
// every report would arrive empty. The visitor's browser is signed in to nothing: the report goes
// on as it came, with none of their headers, and the answer is always empty, to any site.

const REPORT_PATH = /^\/hooks\/sites\/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})$/;
const MAX_REPORT_BYTES = 2048;

/**
 * Handles `POST /hooks/sites/:projectId`; returns false for every other request.
 * @param {import('node:http').IncomingMessage} req
 * @param {import('node:http').ServerResponse} res
 * @param {{ apiUrl?: string }} [options]
 */
export function handleSiteReport(req, res, options = {}) {
	const match = (req.url ?? '').split('?')[0].match(REPORT_PATH);
	if (req.method !== 'POST' || !match) return false;

	const answer = () => {
		if (!res.headersSent) res.writeHead(204, { 'access-control-allow-origin': '*' }).end();
	};
	/** @type {Buffer[]} */
	const chunks = [];
	let size = 0;
	let tooBig = false;
	req.on('data', (/** @type {Buffer} */ chunk) => {
		size += chunk.length;
		if (size > MAX_REPORT_BYTES) tooBig = true;
		else chunks.push(chunk);
	});
	req.on('error', answer);
	req.on('end', () => {
		answer();
		if (tooBig || size === 0) return;
		const body = Buffer.concat(chunks);
		const target = new URL(`/api/hooks/sites/${match[1]}`, options.apiUrl ?? process.env.API_URL ?? 'http://localhost:8080');
		const send = target.protocol === 'https:' ? httpsRequest : httpRequest;
		const upstream = send(target, { method: 'POST', headers: { 'content-length': String(body.length) } }, (response) => response.resume());
		// A report that can't be passed on is let go: the visitor never waits on Nomi.
		upstream.on('error', () => {});
		upstream.end(body);
	});
	return true;
}
