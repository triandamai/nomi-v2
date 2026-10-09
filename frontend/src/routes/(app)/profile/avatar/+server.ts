import type { RequestHandler } from './$types';
import { apiUrl } from '$lib/server/api';

// A new profile picture streams through to the backend, which stores it and saves it on the
// profile (in production, server.js's upload-proxy does the same before SvelteKit's body limit
// applies; this route serves `vite dev`). A 401 goes back so the browser can refresh and retry.
export const POST: RequestHandler = async ({ request, cookies, fetch, url }) => {
	const origin = request.headers.get('origin');
	if (origin && origin !== url.origin) return new Response(null, { status: 403 });
	const accessToken = cookies.get('access_token');
	if (!accessToken) return new Response(null, { status: 401 });
	const response = await fetch(apiUrl('/api/profile/avatar'), {
		method: 'POST',
		headers: { Authorization: `Bearer ${accessToken}`, 'Content-Type': request.headers.get('content-type') ?? 'application/octet-stream' },
		body: request.body,
		duplex: 'half',
	} as RequestInit);
	return new Response(response.body, { status: response.status, headers: { 'Content-Type': response.headers.get('Content-Type') ?? 'application/json' } });
};
