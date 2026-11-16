import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { RequestHandler } from './$types';

// Feedback on one version of a chat's design previews, from the Design Studio.
export const POST: RequestHandler = async ({ params, request, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(params.sessionId)}/design/feedback`, {
		method: 'POST',
		body: JSON.stringify(await request.json()),
	});
	if (!response.ok) throw error(response.status, (await response.text()) || m.studio_feedback_failed());
	return json(await response.json(), { status: 201 });
};
