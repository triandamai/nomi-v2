import type { RequestHandler } from './$types';
import { apiFetch } from '$lib/server/api';

// Refreshes an expired session before an upload is retried (see uploadFile in $lib/attachments).
export const POST: RequestHandler = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/whoami');
	return new Response(null, { status: response.ok ? 204 : 401 });
};
