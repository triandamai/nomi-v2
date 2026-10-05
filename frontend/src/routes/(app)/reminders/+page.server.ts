import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Reminder } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/reminders');
	if (!response.ok) return { reminders: null };
	const reminders = (await response.json()) as { timezone: string; upcoming: Reminder[]; past: Reminder[] };
	return { reminders };
};

export const actions: Actions = {
	create: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const label = String(data.get('label') ?? '').trim();
		const runAt = String(data.get('run_at') ?? '');
		const recurrence = String(data.get('recurrence') ?? 'once');
		if (!label) return fail(400, { error: 'Say what to remind you about.' });
		if (!runAt) return fail(400, { error: 'Pick a date and time.' });
		const response = await apiFetch(fetch, cookies, '/api/reminders', {
			method: 'POST',
			body: JSON.stringify({ label, run_at: runAt, recurrence: recurrence === 'once' ? null : recurrence }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message.includes('future') ? 'Pick a time in the future.' : 'Couldn’t save that reminder.' });
		}
		return { created: true };
	},
	cancel: async ({ request, cookies, fetch }) => {
		const id = String((await request.formData()).get('id') ?? '');
		const response = await apiFetch(fetch, cookies, `/api/reminders/${encodeURIComponent(id)}/cancel`, { method: 'POST' });
		if (!response.ok && response.status !== 404) return fail(response.status, { error: 'Couldn’t cancel that reminder.' });
		return { cancelled: id };
	},
};
