import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const inbox = await import('./inbox');

const page = (ids: string[], extra: Record<string, unknown> = {}) => ({
	items: ids.map((id) => ({ id, kind: 'account', title: id, body: '', link: null, read: false, created_at: '2026-10-21T00:00:00Z' })),
	unread: 3,
	page: 1,
	pages: 2,
	page_size: 20,
	total: 25,
	counts: { all: 25, unread: 3, read: 22, promo: 0 },
	...extra,
});
const reminders = { timezone: 'Asia/Jakarta', upcoming: [], past: [], scheduled_tasks: [] };

function eventWith(answer: Parameters<typeof fakeBackend>[0]) {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
}

describe('the inbox on the server', () => {
	it('loads what’s unread, the pages shown so far, the reminders and the email settings', async () => {
		const { backend, event } = eventWith((call) => {
			if (call.url.includes('filter=unread')) return json(page(['u1', 'u2']));
			if (call.url.includes('page=2')) return json(page(['p2'], { page: 2 }));
			if (call.url.includes('/api/notifications?page=1')) return json(page(['u1', 'p1']));
			if (call.url.endsWith('/api/reminders')) return json(reminders);
			return json({ email_account: true, email_promos: false });
		});
		const data = await inbox.loadInbox(event, 2);
		expect(backend.calls.map((call) => call.url.replace('http://api.test', '')).sort()).toEqual([
			'/api/notifications/preferences',
			'/api/notifications?filter=unread',
			'/api/notifications?page=1',
			'/api/notifications?page=2',
			'/api/reminders',
		]);
		expect(data).toMatchObject({ counts: { all: 25, unread: 3 }, pages: 2, shown: 2, reminders, preferences: { email_account: true, email_promos: false } });
		expect(data?.notifications.map((item) => item.id)).toEqual(['u1', 'u2', 'u1', 'p1', 'p2']);
	});

	it('loads without the email settings when only they fail', async () => {
		const { event } = eventWith((call) => (call.url.includes('preferences') ? new Response('', { status: 500 }) : call.url.endsWith('/api/reminders') ? json(reminders) : json(page([]))));
		expect(await inbox.loadInbox(event)).toMatchObject({ preferences: null, shown: 1 });
	});

	it('says it couldn’t load when the notifications or the reminders fail', async () => {
		const down = eventWith((call) => (call.url.endsWith('/api/reminders') ? new Response('', { status: 500 }) : json(page([]))));
		expect(await inbox.loadInbox(down.event)).toBeNull();
		const unreachable = eventWith(() => 'unreachable');
		expect(await inbox.loadInbox(unreachable.event)).toBeNull();
	});

	it('marks one notification read, or all of them', async () => {
		const { backend, event } = eventWith(() => new Response(null, { status: 204 }));
		expect(await inbox.markRead(event, 'n 1')).toEqual({ ok: true, data: undefined });
		expect(await inbox.markAllRead(event)).toEqual({ ok: true, data: undefined });
		expect(backend.calls.map((call) => [call.method, call.url])).toEqual([
			['POST', 'http://api.test/api/notifications/n%201/read'],
			['POST', 'http://api.test/api/notifications/read-all'],
		]);
	});

	it('sets a reminder at a local time, which the backend reads in the person’s zone', async () => {
		const { backend, event } = eventWith(() => json({ id: 'r1', title: 'Pay PLN' }, 201));
		expect(await inbox.remind(event, { title: 'Pay PLN', due_at: '2026-10-25T10:00', recurrence: 'monthly' })).toMatchObject({ ok: true, data: { id: 'r1' } });
		expect(backend.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/reminders', body: { title: 'Pay PLN', due_at: '2026-10-25T10:00', recurrence: 'monthly' } });
	});

	it('passes on what the backend says when it won’t set one', async () => {
		const { event } = eventWith(() => new Response('due_at must be in the future', { status: 400 }));
		expect(await inbox.remind(event, { title: 'Late', due_at: '2020-01-01T10:00', recurrence: null })).toEqual({ ok: false, status: 400, message: 'due_at must be in the future' });
	});

	it('ticks off, cancels or snoozes a reminder, and cancels scheduled work', async () => {
		const { backend, event } = eventWith(() => new Response(null, { status: 204 }));
		await inbox.actOnReminder(event, 'r1', 'snooze', 60);
		await inbox.actOnReminder(event, 'r1', 'done');
		await inbox.cancelTask(event, 't1');
		expect(backend.calls.map((call) => [call.url, call.body])).toEqual([
			['http://api.test/api/reminders/r1', { action: 'snooze', minutes: 60 }],
			['http://api.test/api/reminders/r1', { action: 'done' }],
			['http://api.test/api/scheduled-tasks/t1/cancel', undefined],
		]);
	});

	it('saves which emails the person gets', async () => {
		const { backend, event } = eventWith(() => json({ email_account: false, email_promos: true }));
		expect(await inbox.savePreferences(event, { email_account: false, email_promos: true })).toEqual({ ok: true, data: { email_account: false, email_promos: true } });
		expect(backend.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/notifications/preferences', body: { email_account: false, email_promos: true } });
	});
});
