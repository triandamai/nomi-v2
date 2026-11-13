import { error, fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import * as inbox from '$lib/server/inbox';
import type { Outcome } from '$lib/server/chat';
import { agenda, filterCounts, inboxEntries } from '$lib/inbox/feed';
import type { Actions, PageServerLoad } from './$types';

/**
 * The Inbox (App / Inbox BFoiZ): notifications and reminders together. It reloads with the shell
 * (`app:shell`: a reminder going off posts in its chat) and when a notification arrives
 * (`app:inbox`). `?shown=` keeps the earlier pages of notifications shown so far.
 */
export const load: PageServerLoad = async (event) => {
	event.depends('app:shell', 'app:inbox');
	const data = await inbox.loadInbox(event, Number(event.url.searchParams.get('shown')) || 1);
	if (!data) error(503, { message: m.inbox_load_failed() });
	const now = new Date();
	const entries = inboxEntries(data.notifications, data.reminders);
	const needs = filterCounts(entries, data.counts).needs;
	const coming = agenda(entries, now, data.reminders.timezone).reduce((sum, day) => sum + day.entries.length, 0);
	return { title: m.nav_inbox(), meta: m.inbox_meta({ needs, coming }), now: now.toISOString(), ...data };
};

const REPEATS = ['daily', 'weekly', 'monthly'];
const REMINDER_ACTIONS = ['done', 'cancel', 'snooze'] as const;
const LOCAL_TIME = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/;

const field = (form: FormData, name: string) => String(form.get(name) ?? '').trim();
const refused = (outcome: Extract<Outcome<unknown>, { ok: false }>) => fail(outcome.status, { message: outcome.message });

export const actions: Actions = {
	read: async (event) => {
		const id = field(await event.request.formData(), 'id');
		const done = await inbox.markRead(event, id);
		return done.ok ? { read: id } : refused(done);
	},
	readAll: async (event) => {
		const done = await inbox.markAllRead(event);
		return done.ok ? { readAll: true } : refused(done);
	},
	/** From the quick line, read in the browser: a title, a local date and time, and how it repeats. */
	remind: async (event) => {
		const form = await event.request.formData();
		const [title, due_at, repeat] = [field(form, 'title'), field(form, 'due_at'), field(form, 'repeat')];
		if (!title) return fail(400, { field: 'title', message: m.inbox_remind_what() });
		if (!LOCAL_TIME.test(due_at) || (repeat && !REPEATS.includes(repeat))) return fail(400, { field: 'when', message: m.inbox_remind_when() });
		const done = await inbox.remind(event, { title, due_at, recurrence: repeat || null });
		if (!done.ok) return done.message.includes('future') ? fail(done.status, { field: 'when', message: m.inbox_remind_past() }) : refused(done);
		return { reminded: { title, due_at } };
	},
	reminder: async (event) => {
		const form = await event.request.formData();
		const action = field(form, 'do') as (typeof REMINDER_ACTIONS)[number];
		if (!REMINDER_ACTIONS.includes(action)) return fail(400, { message: m.inbox_action_failed() });
		const minutes = action === 'snooze' ? Number(field(form, 'minutes')) || undefined : undefined;
		const done = await inbox.actOnReminder(event, field(form, 'id'), action, minutes);
		return done.ok ? { reminder: action } : refused(done);
	},
	cancelTask: async (event) => {
		const id = field(await event.request.formData(), 'id');
		const done = await inbox.cancelTask(event, id);
		return done.ok ? { cancelledTask: id } : refused(done);
	},
	preferences: async (event) => {
		const form = await event.request.formData();
		const done = await inbox.savePreferences(event, { email_account: field(form, 'email_account') === 'true', email_promos: field(form, 'email_promos') === 'true' });
		return done.ok ? { preferences: done.data } : refused(done);
	},
};
