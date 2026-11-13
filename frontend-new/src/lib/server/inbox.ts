import { isRedirect } from '@sveltejs/kit';
import type { EmailPreferences, NotificationItem, NotificationPage, ReminderItem, RemindersList } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';
import { readApiError } from './apiError';
import type { Outcome } from './chat';

// The Inbox's calls to the backend, as the person: their notifications (what's unread, and the
// pages shown so far, 20 each), Kala's reminders and the work agents will do later, and which
// emails they get; marking read, setting, ticking off, snoozing and cancelling. Logic and API use
// from the old app (frontend/src/routes/(app)/{notifications,reminders}).

/** How many pages of notifications the Inbox shows at most, 20 each. */
export const MAX_PAGES = 10;

export type InboxData = {
	/** The unread ones first, then every page shown; an unread one can be in both. */
	notifications: NotificationItem[];
	counts: { all: number; unread: number };
	pages: number;
	shown: number;
	reminders: RemindersList;
	/** Null when they couldn't be loaded; the rest of the Inbox still shows. */
	preferences: EmailPreferences | null;
};

/** A backend that can't be reached answers like one that's down. */
async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

async function read<T>(event: ApiEvent, path: string): Promise<T | null> {
	const response = await call(event, path);
	return response.ok ? ((await response.json()) as T) : null;
}

async function send<T>(event: ApiEvent, path: string, method: string, body?: unknown, parse = false): Promise<Outcome<T>> {
	const response = await call(event, path, { method, ...(body !== undefined && { body: JSON.stringify(body) }) });
	if (!response.ok) return { ok: false, status: response.status, message: (await readApiError(response)).message };
	return { ok: true, data: (parse ? await response.json() : undefined) as T };
}

export async function loadInbox(event: ApiEvent, shown = 1): Promise<InboxData | null> {
	const pages = Math.min(Math.max(1, Math.floor(shown)), MAX_PAGES);
	const [unread, reminders, preferences, ...all] = await Promise.all([
		read<NotificationPage>(event, '/api/notifications?filter=unread'),
		read<RemindersList>(event, '/api/reminders'),
		read<EmailPreferences>(event, '/api/notifications/preferences'),
		...Array.from({ length: pages }, (_, index) => read<NotificationPage>(event, `/api/notifications?page=${index + 1}`)),
	]);
	if (!unread || !reminders || all.some((page) => !page)) return null;
	const first = all[0]!;
	return {
		notifications: [unread, ...all].flatMap((page) => page!.items),
		counts: { all: first.counts.all, unread: first.counts.unread },
		pages: first.pages,
		shown: pages,
		reminders,
		preferences,
	};
}

export const markRead = (event: ApiEvent, id: string) => send<void>(event, `/api/notifications/${encodeURIComponent(id)}/read`, 'POST');

export const markAllRead = (event: ApiEvent) => send<void>(event, '/api/notifications/read-all', 'POST');

/** `due_at` is local ("2026-10-25T10:00"): the backend reads it in the person's zone. */
export const remind = (event: ApiEvent, reminder: { title: string; due_at: string; recurrence: string | null }) =>
	send<ReminderItem>(event, '/api/reminders', 'POST', reminder, true);

export const actOnReminder = (event: ApiEvent, id: string, action: 'done' | 'cancel' | 'snooze', minutes?: number) =>
	send<void>(event, `/api/reminders/${encodeURIComponent(id)}`, 'POST', minutes === undefined ? { action } : { action, minutes });

export const cancelTask = (event: ApiEvent, id: string) => send<void>(event, `/api/scheduled-tasks/${encodeURIComponent(id)}/cancel`, 'POST');

export const savePreferences = (event: ApiEvent, preferences: EmailPreferences) =>
	send<EmailPreferences>(event, '/api/notifications/preferences', 'PUT', preferences, true);
