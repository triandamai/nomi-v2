import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// A profile picture, read through the API (it lives in the bucket or on the server's disk; the
// browser never needs the bucket to be public).
export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/avatars/${encodeURIComponent(params.userId)}/${encodeURIComponent(params.file)}`);
	if (!response.ok) throw error(response.status === 401 ? 401 : 404, 'Not found');
	return new Response(response.body, {
		headers: {
			'content-type': response.headers.get('content-type') ?? 'application/octet-stream',
			'cache-control': response.headers.get('cache-control') ?? 'private, max-age=3600',
			'x-content-type-options': 'nosniff',
		},
	});
};
