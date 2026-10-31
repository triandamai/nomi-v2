import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { RequestHandler } from './$types';

const path = (sessionId: string, shareId: string) => `/api/sessions/${encodeURIComponent(sessionId)}/shares/${encodeURIComponent(shareId)}`;

// The same link with other settings.
export const PUT: RequestHandler = async ({ params, request, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, path(params.sessionId, params.shareId), { method: 'PUT', body: JSON.stringify(await request.json()) });
	if (!response.ok) throw error(response.status, m.share_failed());
	return json(await response.json());
};

// The link stops working.
export const DELETE: RequestHandler = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, path(params.sessionId, params.shareId), { method: 'DELETE' });
	if (!response.ok) throw error(response.status, m.share_failed());
	return new Response(null, { status: 204 });
};
