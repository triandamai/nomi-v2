import { describe, expect, it } from 'vitest';
import type { NotificationItem, ReminderItem, RemindersList, ScheduledTask } from '$lib/adapters/api';
import { agenda, entryCaption, entryCrew, entryTag, entryTime, filterCounts, inboxEntries, inboxFeed, type InboxEntry } from './feed';

// Wed 21 Oct 2026, 08:14 in Jakarta (UTC+7), as the pen's Inbox has it.
const now = new Date('2026-10-21T01:14:00Z');
const zone = 'Asia/Jakarta';

const notification = (id: string, extra: Partial<NotificationItem> = {}): NotificationItem => ({
	id,
	kind: 'account',
	title: id,
	body: `${id} body`,
	link: null,
	read: true,
	created_at: '2026-10-21T00:00:00Z',
	...extra,
});

const reminder = (id: string, due_at: string, extra: Partial<ReminderItem> = {}): ReminderItem => ({
	id,
	session_id: 'kala-chat',
	title: id,
	notes: null,
	due_at,
	recurrence: null,
	recurrence_weekday: null,
	recurrence_day_of_month: null,
	status: 'active',
	created_by: 'user',
	last_fired_at: null,
	created_at: '2026-10-01T00:00:00Z',
	...extra,
});

const task: ScheduledTask = { id: 't1', label: 'Sum up my spending', run_at: '2026-10-22T02:00:00Z', recurrence: null, agent: 'money', session_id: 'dana-chat' };

const list: RemindersList = {
	timezone: zone,
	upcoming: [
		reminder('stretch', '2026-10-21T09:00:00Z'),
		reminder('call', '2026-10-21T13:00:00Z', { notes: 'Ask about the Bandung trip' }),
		reminder('pln', '2026-10-25T03:00:00Z', { recurrence: 'monthly', recurrence_day_of_month: 25 }),
		reminder('passport', '2026-12-01T03:00:00Z'),
	],
	past: [
		reminder('netflix', '2026-10-21T00:40:00Z', { status: 'fired', last_fired_at: '2026-10-21T00:40:05Z' }),
		reminder('gym', '2026-10-20T23:30:00Z', { status: 'done', recurrence: 'weekly', recurrence_weekday: 1 }),
		reminder('old', '2026-10-15T03:00:00Z', { status: 'cancelled' }),
	],
	scheduled_tasks: [task],
};

const notifications = [
	notification('invite', { read: false, created_at: '2026-10-18T03:00:00Z' }),
	notification('deployed', { kind: 'deploy', created_at: '2026-10-21T00:02:00Z', link: '/projects/p1' }),
	notification('promo', { kind: 'promo', created_at: '2026-10-19T03:00:00Z' }),
];

const entries = inboxEntries([...notifications, notifications[0]], list);
const ids = (list: InboxEntry[]) => list.map((entry) => entry.id);

describe('the inbox’s entries', () => {
	it('puts notifications, reminders and scheduled work together, each notification once', () => {
		expect(ids(entries).sort()).toEqual(['call', 'deployed', 'gym', 'invite', 'netflix', 'old', 'passport', 'pln', 'promo', 'stretch', 't1']);
	});
});

describe('the inbox’s feed', () => {
	it('shows what needs you, today around now, and the earlier ones (All)', () => {
		const feed = inboxFeed(entries, 'all', now, zone);
		expect(ids(feed.needs)).toEqual(['netflix', 'invite']);
		expect(ids(feed.today)).toEqual(['gym', 'deployed', 'stretch', 'call']);
		expect(feed.nowAt).toBe(2);
		expect(feed.later).toEqual([]);
		expect(ids(feed.earlier)).toEqual(['promo', 'old']);
	});

	it('shows only what needs you (Needs you)', () => {
		const feed = inboxFeed(entries, 'needs', now, zone);
		expect(ids(feed.needs)).toEqual(['netflix', 'invite']);
		expect([feed.today, feed.later, feed.earlier]).toEqual([[], [], []]);
	});

	it('shows every reminder and scheduled task, the days ahead too (Reminders)', () => {
		const feed = inboxFeed(entries, 'reminders', now, zone);
		expect(ids(feed.needs)).toEqual(['netflix']);
		expect(ids(feed.today)).toEqual(['gym', 'stretch', 'call']);
		expect(feed.nowAt).toBe(1);
		expect(feed.later.map((day) => [day.day, ids(day.entries)])).toEqual([
			['2026-10-22', ['t1']],
			['2026-10-25', ['pln']],
			['2026-12-01', ['passport']],
		]);
		expect(ids(feed.earlier)).toEqual(['old']);
	});

	it('shows only notifications (Updates)', () => {
		const feed = inboxFeed(entries, 'updates', now, zone);
		expect([ids(feed.needs), ids(feed.today), ids(feed.earlier)]).toEqual([['invite'], ['deployed'], ['promo']]);
	});

	it('lists the next 7 days of reminders and scheduled work, by day', () => {
		expect(agenda(entries, now, zone).map((day) => [day.day, ids(day.entries)])).toEqual([
			['2026-10-21', ['stretch', 'call']],
			['2026-10-22', ['t1']],
			['2026-10-25', ['pln']],
		]);
	});

	it('counts each filter: unread notifications and fired reminders need you; All is every update and reminder', () => {
		expect(filterCounts(entries, { all: 30, unread: 4 })).toEqual({ all: 38, needs: 5, reminders: 8, updates: 30 });
	});
});

describe('an entry, as a row says it', () => {
	const find = (id: string) => entries.find((entry) => entry.id === id)!;

	it('is from the crew member it belongs to', () => {
		expect(['invite', 'deployed', 'stretch', 't1'].map((id) => entryCrew(find(id)))).toEqual(['nomi', 'koda', 'kala', 'dana']);
	});

	it('is tagged with what it is in mono', () => {
		expect(entryTag(find('invite'), 'en')).toBe('account');
		expect(entryTag(find('deployed'), 'en')).toBe('deploy');
		expect(entryTag(find('stretch'), 'en')).toBe('reminder · once');
		expect(entryTag(find('gym'), 'en')).toBe('reminder · every Mon');
		expect(entryTag(find('pln'), 'en')).toBe('reminder · every month, day 25');
		expect(entryTag(find('t1'), 'en')).toBe('scheduled · Dana');
	});

	it('says when an upcoming one is due, and how a finished one ended', () => {
		expect(entryCaption(find('stretch'), now)).toBe('in 7h 46m');
		expect(entryCaption(find('call'), now)).toBe('in 11h 46m · Ask about the Bandung trip');
		expect(entryCaption(find('gym'), now)).toBe('Done');
		expect(entryCaption(find('old'), now)).toBe('Cancelled');
		expect(entryCaption(find('deployed'), now)).toBe('deployed body');
		expect(entryCaption(find('t1'), now)).toBe('in 1d · Dana will do this');
	});

	it('shows its clock time today, and its date before', () => {
		expect(entryTime(find('deployed'), now, zone, 'en')).toBe('07:02');
		expect(entryTime(find('netflix'), now, zone, 'en')).toBe('07:40');
		expect(entryTime(find('invite'), now, zone, 'en')).toBe('18 Oct');
	});
});
