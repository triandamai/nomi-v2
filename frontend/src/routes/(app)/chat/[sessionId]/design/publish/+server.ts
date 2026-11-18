import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { RequestHandler } from './$types';

// Publishing one version of a chat's design previews again, from the Design Studio, after it
// couldn't be. The answer says where it is now, or why it still isn't.
export const POST: RequestHandler = async ({ params, request, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(params.sessionId)}/design/publish`, {
		method: 'POST',
		body: JSON.stringify(await request.json()),
	});
	if (!response.ok) throw error(response.status, m.studio_publish_failed());
	return json(await response.json());
};
