import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

// A file attached in a chat, for `vite dev`: in production server.js streams it to the backend
// before SvelteKit sees it (ws-proxy/upload-proxy.js, the same contract). The raw file is the body,
// its name %-encoded in X-File-Name; a 401 goes back so the browser can renew its sign-in
// (./session) and try again.
const PASSED = ['content-type', 'content-length', 'x-file-name', 'x-attachment-kind', 'x-transcript'];

export const POST: RequestHandler = async ({ request, cookies, fetch, url }) => {
	const origin = request.headers.get('origin');
	if (origin && origin !== url.origin) return new Response(null, { status: 403 });
	const accessToken = cookies.get('access_token');
	if (!accessToken) return new Response(null, { status: 401 });
	const headers = new Headers({ authorization: `Bearer ${accessToken}` });
	for (const name of PASSED) {
		const value = request.headers.get(name);
		if (value) headers.set(name, value);
	}
	const response = await fetch(apiUrl('/api/attachments'), { method: 'POST', headers, body: request.body, duplex: 'half' } as RequestInit);
	return new Response(response.body, { status: response.status, headers: { 'content-type': response.headers.get('content-type') ?? 'application/json' } });
};
