import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { RequestHandler } from './$types';

// The crew's reply, asked again: the same, shorter, longer or as a list (RegenerateMenu).
export const POST: RequestHandler = async ({ params, request, cookies, fetch }) => {
	const { style } = (await request.json()) as { style: string };
	const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(params.sessionId)}/messages/${encodeURIComponent(params.messageId)}/regenerate`, {
		method: 'POST',
		body: JSON.stringify({ style }),
	});
	if (!response.ok) throw error(response.status, m.msg_rerun_failed());
	return json(await response.json());
};
