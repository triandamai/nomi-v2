import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { RequestHandler } from './$types';

// The person's approval of one version of a chat's design previews, from the Design Studio.
export const POST: RequestHandler = async ({ params, request, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(params.sessionId)}/design/approval`, {
		method: 'POST',
		body: JSON.stringify(await request.json()),
	});
	if (!response.ok) throw error(response.status, m.studio_approve_failed());
	return json(await response.json(), { status: 201 });
};
