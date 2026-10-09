import type { RequestHandler } from './$types';
import { apiUrl } from '$lib/server/api';
import { UPLOAD_HEADERS } from '$lib/server/attachments';

// Uploads stream through to the backend as they arrive (in production, server.js's
// upload-proxy does the same before SvelteKit's body limit applies; this route serves `vite dev`).
// A 401 goes back to the browser, which refreshes the session (./session) and tries again.
export const POST: RequestHandler = async ({ request, cookies, fetch, url }) => {
	// Cookies ride along on cross-site requests; an upload must come from Nomi's own pages.
	const origin = request.headers.get('origin');
	if (origin && origin !== url.origin) return new Response(null, { status: 403 });
	const accessToken = cookies.get('access_token');
	if (!accessToken) return new Response(null, { status: 401 });
	const headers = new Headers({ Authorization: `Bearer ${accessToken}` });
	for (const name of UPLOAD_HEADERS) {
		const value = request.headers.get(name);
		if (value) headers.set(name, value);
	}
	const response = await fetch(apiUrl('/api/attachments'), {
		method: 'POST',
		headers,
		body: request.body,
		// Node's fetch needs this to send a streamed body.
		duplex: 'half',
	} as RequestInit);
	return new Response(response.body, { status: response.status, headers: { 'Content-Type': response.headers.get('Content-Type') ?? 'application/json' } });
};
