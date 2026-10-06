import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';
import { m } from '$lib/paraglide/messages';

// Done / Snooze from a reminder's bubble in chat (ReminderBlock).
export const POST: RequestHandler = async ({ params, request, cookies, fetch }) => {
	const { action, minutes } = (await request.json()) as { action: string; minutes?: number };
	const response = await apiFetch(fetch, cookies, `/api/reminders/${encodeURIComponent(params.id)}`, {
		method: 'POST',
		body: JSON.stringify({ action, minutes }),
	});
	if (!response.ok) throw error(response.status, m.rem_update_failed());
	return json({ ok: true });
};
