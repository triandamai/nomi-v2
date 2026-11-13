import { m } from '$lib/paraglide/messages';
import type { NotificationItem, RemindersList } from '$lib/adapters/api';
import { crewOf } from '$lib/adapters/shell';
import { clock, dayKey } from '$lib/chat/dates';
import { CREW, type CrewId } from '$lib/ds/agents/crew';

// The Inbox (App / Inbox BFoiZ): the person's notifications and Kala's reminders together, with
// the work agents will do later beside them. What needs the person comes first (an unread
// notification, a reminder that went off and wasn't ticked off), then today around the now line,
// then the earlier ones; the Reminders filter also lists the days ahead. Everything is placed
// where the person is (their zone), so the server's render and the browser's agree.

export type ReminderStatus = 'active' | 'fired' | 'done' | 'cancelled';

export type InboxEntry =
	| { kind: 'notification'; id: string; title: string; body: string; type: string; link: string | null; read: boolean; at: string }
	| {
			kind: 'reminder';
			id: string;
			title: string;
			notes: string | null;
			status: ReminderStatus;
			repeat: string | null;
			weekday: number | null;
			monthDay: number | null;
			/** When it's due, or when it went off once it has. */
			at: string;
			sessionId: string;
	  }
	| { kind: 'task'; id: string; title: string; agent: string; at: string; sessionId: string };

export type Filter = 'all' | 'needs' | 'reminders' | 'updates';
export type Day = { day: string; entries: InboxEntry[] };
/** `nowAt`: how many of today's entries come before the now line. */
export type Feed = { needs: InboxEntry[]; today: InboxEntry[]; nowAt: number; later: Day[]; earlier: InboxEntry[] };

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/** Notifications (from more than one page, so each only once), reminders and scheduled work. */
export function inboxEntries(notifications: NotificationItem[], reminders: RemindersList): InboxEntry[] {
	const seen = new Set<string>();
	const entries: InboxEntry[] = [];
	for (const item of notifications) {
		if (seen.has(item.id)) continue;
		seen.add(item.id);
		entries.push({ kind: 'notification', id: item.id, title: item.title, body: item.body, type: item.kind, link: item.link, read: item.read, at: item.created_at });
	}
	for (const item of [...reminders.upcoming, ...reminders.past]) {
		const status = item.status as ReminderStatus;
		entries.push({
			kind: 'reminder',
			id: item.id,
			title: item.title,
			notes: item.notes,
			status,
			repeat: item.recurrence,
			weekday: item.recurrence_weekday,
			monthDay: item.recurrence_day_of_month,
			at: status === 'fired' ? (item.last_fired_at ?? item.due_at) : item.due_at,
			sessionId: item.session_id,
		});
	}
	for (const item of reminders.scheduled_tasks) entries.push({ kind: 'task', id: item.id, title: item.label, agent: item.agent, at: item.run_at, sessionId: item.session_id });
	return entries;
}

export const needsYou = (entry: InboxEntry) => (entry.kind === 'notification' ? !entry.read : entry.kind === 'reminder' && entry.status === 'fired');
const finished = (entry: InboxEntry) => entry.kind === 'reminder' && (entry.status === 'done' || entry.status === 'cancelled');
const ahead = (entry: InboxEntry) => entry.kind === 'task' || (entry.kind === 'reminder' && entry.status === 'active');
const shownIn = (filter: Filter, entry: InboxEntry) =>
	filter === 'all' || (filter === 'needs' ? needsYou(entry) : filter === 'updates' ? entry.kind === 'notification' : entry.kind !== 'notification');
const oldestFirst = (a: InboxEntry, b: InboxEntry) => Date.parse(a.at) - Date.parse(b.at);
const newestFirst = (a: InboxEntry, b: InboxEntry) => Date.parse(b.at) - Date.parse(a.at);

function byDay(entries: InboxEntry[], timeZone: string | undefined): Day[] {
	const days = new Map<string, InboxEntry[]>();
	for (const entry of [...entries].sort(oldestFirst)) {
		const key = dayKey(entry.at, timeZone);
		days.set(key, [...(days.get(key) ?? []), entry]);
	}
	return [...days].map(([day, entries]) => ({ day, entries }));
}

/** Where each entry the filter shows goes. Days ahead are listed only under Reminders. */
export function inboxFeed(entries: InboxEntry[], filter: Filter, now: Date, timeZone?: string): Feed {
	const today = dayKey(now, timeZone);
	const needs: InboxEntry[] = [];
	const done: InboxEntry[] = [];
	const coming: InboxEntry[] = [];
	const later: InboxEntry[] = [];
	const earlier: InboxEntry[] = [];
	for (const entry of entries.filter((entry) => shownIn(filter, entry))) {
		const day = dayKey(entry.at, timeZone);
		if (needsYou(entry)) needs.push(entry);
		else if (day < today) earlier.push(entry);
		else if (finished(entry) || Date.parse(entry.at) <= now.getTime()) done.push(entry);
		else if (day === today) coming.push(entry);
		else if (filter === 'reminders') later.push(entry);
	}
	return {
		needs: needs.sort(newestFirst),
		today: [...done.sort(oldestFirst), ...coming.sort(oldestFirst)],
		nowAt: done.length,
		later: byDay(later, timeZone),
		earlier: earlier.sort(newestFirst),
	};
}

/** Coming up: reminders and scheduled work still ahead today and in the six days after. */
export function agenda(entries: InboxEntry[], now: Date, timeZone?: string, days = 7): Day[] {
	const last = dayKey(new Date(now.getTime() + (days - 1) * DAY), timeZone);
	return byDay(
		entries.filter((entry) => ahead(entry) && Date.parse(entry.at) > now.getTime() && dayKey(entry.at, timeZone) <= last),
		timeZone,
	);
}

/** Each filter's count: All is every update and reminder; unread ones and fired reminders need you. */
export function filterCounts(entries: InboxEntry[], notifications: { all: number; unread: number }): Record<Filter, number> {
	const reminders = entries.filter((entry) => entry.kind !== 'notification').length;
	const fired = entries.filter((entry) => entry.kind === 'reminder' && entry.status === 'fired').length;
	return { all: notifications.all + reminders, needs: notifications.unread + fired, reminders, updates: notifications.all };
}

/** Whose it is: Kala keeps reminders, Koda the projects; scheduled work is its agent's. */
export function entryCrew(entry: InboxEntry): CrewId {
	if (entry.kind === 'reminder') return 'kala';
	if (entry.kind === 'task') return crewOf(entry.agent) ?? 'nomi';
	return entry.type === 'deploy' || entry.type === 'project' ? 'koda' : 'nomi';
}

const intlLocale = (language: string) => (language === 'id' ? 'id-ID' : 'en-GB');

/** A weekday's short name (0 is Sunday), in the person's language. */
function weekdayName(weekday: number, language: string): string {
	return new Intl.DateTimeFormat(intlLocale(language), { weekday: 'short', timeZone: 'UTC' }).format(Date.UTC(2026, 9, 18 + weekday));
}

const TAGS: Record<string, () => string> = {
	account: m.inbox_tag_account,
	subscription: m.inbox_tag_subscription,
	quota: m.inbox_tag_quota,
	promo: m.inbox_tag_promo,
	deploy: m.inbox_tag_deploy,
	project: m.inbox_tag_project,
};

/** The mono tag after the title: what kind of update, how a reminder repeats, whose work it is. */
export function entryTag(entry: InboxEntry, language: string): string {
	if (entry.kind === 'notification') return (TAGS[entry.type] ?? m.inbox_tag_update)();
	if (entry.kind === 'task') return m.inbox_tag_task({ name: crewName(entry) });
	if (entry.repeat === 'daily') return m.inbox_tag_reminder_daily();
	if (entry.repeat === 'weekly' && entry.weekday !== null) return m.inbox_tag_reminder_weekly({ day: weekdayName(entry.weekday, language) });
	if (entry.repeat === 'monthly' && entry.monthDay !== null) return m.inbox_tag_reminder_monthly({ day: entry.monthDay });
	return m.inbox_tag_reminder_once();
}

function crewName(entry: InboxEntry): string {
	const id = entryCrew(entry);
	return CREW.find((member) => member.id === id)?.name ?? '';
}

/** How long until `ms` from now, coarse: "in 7h 46m", "in 12m", "in 2d". */
export function until(ms: number): string {
	const span =
		ms >= DAY
			? m.time_days({ count: Math.floor(ms / DAY) })
			: [ms >= HOUR && m.time_hours({ count: Math.floor(ms / HOUR) }), m.time_minutes({ count: Math.max(1, Math.floor((ms % HOUR) / MINUTE)) })].filter(Boolean).join(' ');
	return m.inbox_in({ span });
}

/** The line under the title: when an upcoming one is due, how a finished one ended, what an update says. */
export function entryCaption(entry: InboxEntry, now: Date): string {
	if (entry.kind === 'notification') return entry.body;
	const left = Date.parse(entry.at) - now.getTime();
	const when = entry.kind === 'task' || entry.status === 'active' ? (left > 0 ? until(left) : undefined) : undefined;
	if (entry.kind === 'task') return [when, m.inbox_task_caption({ name: crewName(entry) })].filter(Boolean).join(' · ');
	const ended = entry.status === 'done' ? m.inbox_done() : entry.status === 'cancelled' ? m.inbox_cancelled() : undefined;
	return [when ?? ended, entry.notes].filter(Boolean).join(' · ');
}

/** The time column: the clock time for today and the days ahead, the date for earlier days. */
export function entryTime(entry: InboxEntry, now: Date, timeZone: string | undefined, language: string): string {
	if (dayKey(entry.at, timeZone) >= dayKey(now, timeZone)) return clock(entry.at, timeZone);
	return new Intl.DateTimeFormat(intlLocale(language), { day: 'numeric', month: 'short', timeZone: timeZone ?? 'UTC' }).format(new Date(entry.at));
}

/** A day's name in the feed ("Wed 21 Oct") and in Coming up ("Thu 22"), in the person's language. */
export function dayName(day: string, language: string, withMonth = true): string {
	const parts = new Intl.DateTimeFormat(intlLocale(language), { weekday: 'short', day: 'numeric', month: 'short', timeZone: 'UTC' }).formatToParts(new Date(`${day}T00:00:00Z`));
	const get = (type: string) => parts.find((part) => part.type === type)?.value ?? '';
	return [get('weekday'), get('day'), withMonth && get('month')].filter(Boolean).join(' ');
}
