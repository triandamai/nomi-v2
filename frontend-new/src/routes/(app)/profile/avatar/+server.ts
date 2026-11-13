import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

// A new profile picture, for `vite dev`: in production server.js streams it to the backend before
// SvelteKit sees it (ws-proxy/upload-proxy.js, the same contract). The raw file is the body; a 401
// goes back so the browser can renew its sign-in and try again.
export const POST: RequestHandler = async ({ request, cookies, fetch, url }) => {
	const origin = request.headers.get('origin');
	if (origin && origin !== url.origin) return new Response(null, { status: 403 });
	const accessToken = cookies.get('access_token');
	if (!accessToken) return new Response(null, { status: 401 });
	const response = await fetch(apiUrl('/api/profile/avatar'), {
		method: 'POST',
		headers: { authorization: `Bearer ${accessToken}`, 'content-type': request.headers.get('content-type') ?? 'application/octet-stream' },
		body: request.body,
		duplex: 'half',
	} as RequestInit);
	return new Response(response.body, { status: response.status, headers: { 'content-type': response.headers.get('content-type') ?? 'application/json' } });
};
