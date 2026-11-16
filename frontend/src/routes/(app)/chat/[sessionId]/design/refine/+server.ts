import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { RequestHandler } from './$types';

// Asking Sora for a new version from the Design Studio: a message in the chat, from the person.
export const POST: RequestHandler = async ({ params, request, cookies, fetch }) => {
	const { text } = (await request.json().catch(() => ({}))) as { text?: unknown };
	if (typeof text !== 'string' || !text.trim()) throw error(400, m.studio_refine_failed());
	const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(params.sessionId)}/messages`, {
		method: 'POST',
		body: JSON.stringify({ text }),
	});
	if (!response.ok) throw error(response.status, m.studio_refine_failed());
	return json({ ok: true }, { status: 202 });
};
