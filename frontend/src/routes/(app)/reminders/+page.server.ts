import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Reminder, ScheduledTask } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/reminders');
	if (!response.ok) return { reminders: null };
	const reminders = (await response.json()) as { timezone: string; upcoming: Reminder[]; past: Reminder[]; scheduled_tasks: ScheduledTask[] };
	return { reminders };
};

async function errorText(response: Response, fallback: string): Promise<string> {
	const text = await response.text();
	if (text.includes('future')) return m.rem_future();
	return text && text.length < 120 ? text : fallback;
}

export const actions: Actions = {
	create: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const title = String(data.get('title') ?? '').trim();
		const dueAt = String(data.get('due_at') ?? '');
		const recurrence = String(data.get('recurrence') ?? 'once');
		const notes = String(data.get('notes') ?? '').trim();
		if (!title) return fail(400, { error: m.rem_say_what() });
		if (!dueAt) return fail(400, { error: m.rem_pick_time() });
		const response = await apiFetch(fetch, cookies, '/api/reminders', {
			method: 'POST',
			body: JSON.stringify({ title, due_at: dueAt, recurrence: recurrence === 'once' ? null : recurrence, notes: notes || null }),
		});
		if (!response.ok) return fail(response.status, { error: await errorText(response, m.rem_save_failed()) });
		return { created: title };
	},
	act: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = String(data.get('id') ?? '');
		const action = String(data.get('action') ?? '');
		const minutes = Number(data.get('minutes') ?? 10);
		const response = await apiFetch(fetch, cookies, `/api/reminders/${encodeURIComponent(id)}`, {
			method: 'POST',
			body: JSON.stringify({ action, minutes }),
		});
		if (!response.ok && response.status !== 404) return fail(response.status, { error: m.rem_update_failed() });
		return { acted: action };
	},
	cancelTask: async ({ request, cookies, fetch }) => {
		const id = String((await request.formData()).get('id') ?? '');
		const response = await apiFetch(fetch, cookies, `/api/scheduled-tasks/${encodeURIComponent(id)}/cancel`, { method: 'POST' });
		if (!response.ok && response.status !== 404) return fail(response.status, { error: m.rem_task_cancel_failed() });
		return { acted: 'cancelTask' };
	},
};
