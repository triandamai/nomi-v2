import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// A recording the crew made with the speak tool, read through the API like avatars.
export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/voice/${encodeURIComponent(params.userId)}/${encodeURIComponent(params.file)}`);
	if (!response.ok) throw error(response.status === 401 ? 401 : 404, 'Not found');
	return new Response(response.body, {
		headers: {
			'content-type': response.headers.get('content-type') ?? 'audio/mpeg',
			'cache-control': response.headers.get('cache-control') ?? 'private, max-age=3600',
			'x-content-type-options': 'nosniff',
		},
	});
};
