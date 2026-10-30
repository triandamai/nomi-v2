import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { RequestHandler } from './$types';

// A new share link for this chat or one of its replies (blocks/twins/ShareTwin).
export const POST: RequestHandler = async ({ params, request, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(params.sessionId)}/shares`, {
		method: 'POST',
		body: JSON.stringify(await request.json()),
	});
	if (!response.ok) throw error(response.status, m.share_failed());
	return json(await response.json(), { status: 201 });
};
