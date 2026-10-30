import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { fetchRenderedMessage } from '$lib/server/fetchRenderedMessage';
import { m } from '$lib/paraglide/messages';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const message = await fetchRenderedMessage(fetch, cookies, params.sessionId, params.messageId);
	return json(message);
};

// The person's message, edited: the crew answers it again (the earlier take stays as a version).
export const PATCH: RequestHandler = async ({ params, request, cookies, fetch }) => {
	const { text } = (await request.json()) as { text: string };
	const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(params.sessionId)}/messages/${encodeURIComponent(params.messageId)}`, {
		method: 'PATCH',
		body: JSON.stringify({ text }),
	});
	if (!response.ok) throw error(response.status, m.msg_rerun_failed());
	return json(await response.json());
};
