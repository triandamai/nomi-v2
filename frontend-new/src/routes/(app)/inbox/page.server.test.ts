import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const soon = (hours: number) => new Date(Date.now() + hours * 3_600_000).toISOString();
const notifications = {
	items: [{ id: 'n1', kind: 'account', title: 'Plan changed', body: '', link: null, read: false, created_at: '2026-10-21T00:00:00Z' }],
	unread: 1,
	page: 1,
	pages: 1,
	page_size: 20,
	total: 1,
	counts: { all: 1, unread: 1, read: 0, promo: 0 },
};
const reminder = (id: string, due_at: string, status = 'active') => ({
	id,
	session_id: 's',
	title: id,
	notes: null,
	due_at,
	recurrence: null,
	recurrence_weekday: null,
	recurrence_day_of_month: null,
	status,
	created_by: 'user',
	last_fired_at: due_at,
	created_at: due_at,
});
const reminders = { timezone: 'UTC', upcoming: [reminder('soon', soon(2)), reminder('later', soon(24 * 30))], past: [reminder('went off', soon(-1), 'fired')], scheduled_tasks: [] };

function backend(answer?: Parameters<typeof fakeBackend>[0]) {
	return fakeBackend(
		answer ??
			((call) => {
				if (call.url.includes('/api/notifications/preferences')) return json({ email_account: true, email_promos: true });
				if (call.url.includes('/api/notifications')) return json(notifications);
				return json(reminders);
			}),
	);
}

const run = (name: keyof typeof actions, form: Record<string, string>, answer: Parameters<typeof fakeBackend>[0] = () => new Response(null, { status: 204 })) => {
	const api = fakeBackend(answer);
	const event = fakeEvent({ url: 'http://localhost/inbox', form, fetch: api.fetch, locals: { accessToken: 'token' } });
	return { api, result: actions[name](event as never) };
};

describe('the Inbox page', () => {
	it('names itself in the top bar with what needs the person and what’s coming up this week', async () => {
		const api = backend();
		const data = (await load(fakeEvent({ url: 'http://localhost/inbox', fetch: api.fetch, locals: { accessToken: 'token' } }) as never)) as Record<string, unknown>;
		expect(data).toMatchObject({ title: 'Inbox', meta: '2 need you · 1 coming up this week', shown: 1, counts: { all: 1, unread: 1 } });
		expect(typeof data.now).toBe('string');
	});

	it('shows more pages of notifications when asked, up to ten', async () => {
		const api = backend();
		await load(fakeEvent({ url: 'http://localhost/inbox?shown=40', fetch: api.fetch, locals: { accessToken: 'token' } }) as never);
		expect(api.calls.filter((call) => /notifications\?page=/.test(call.url))).toHaveLength(10);
	});

	it('shows the error page when the backend can’t give the inbox', async () => {
		const api = backend(() => 'unreachable');
		expect(await thrown(() => load(fakeEvent({ url: 'http://localhost/inbox', fetch: api.fetch, locals: { accessToken: 'token' } }) as never))).toMatchObject({ status: 503 });
	});
});

describe('the Inbox’s actions', () => {
	it('marks a notification read, or all of them', async () => {
		const one = run('read', { id: 'n1' });
		expect(await one.result).toEqual({ read: 'n1' });
		expect(one.api.calls[0].url).toBe('http://api.test/api/notifications/n1/read');
		const all = run('readAll', {});
		expect(await all.result).toEqual({ readAll: true });
		expect(all.api.calls[0].url).toBe('http://api.test/api/notifications/read-all');
	});

	it('sets a reminder from the quick line’s title, local time and repeat', async () => {
		const { api, result } = run('remind', { title: 'Pay PLN', due_at: '2026-10-25T10:00', repeat: 'monthly' }, () => json({ id: 'r1' }, 201));
		expect(await result).toEqual({ reminded: { title: 'Pay PLN', due_at: '2026-10-25T10:00' } });
		expect(api.calls[0].body).toEqual({ title: 'Pay PLN', due_at: '2026-10-25T10:00', recurrence: 'monthly' });
	});

	it('asks what and when before sending a reminder anywhere', async () => {
		const noTitle = run('remind', { title: ' ', due_at: '2026-10-25T10:00' });
		expect(await noTitle.result).toMatchObject({ status: 400, data: { field: 'title', message: 'Say what to remind you about.' } });
		const noTime = run('remind', { title: 'Rent', due_at: '2026-10-25' });
		expect(await noTime.result).toMatchObject({ status: 400, data: { field: 'when' } });
		const badRepeat = run('remind', { title: 'Rent', due_at: '2026-10-25T10:00', repeat: 'hourly' });
		expect(await badRepeat.result).toMatchObject({ status: 400, data: { field: 'when' } });
		expect([noTitle, noTime, badRepeat].flatMap((one) => one.api.calls)).toEqual([]);
	});

	it('says a time already gone plainly', async () => {
		const { result } = run('remind', { title: 'Late', due_at: '2020-01-01T10:00' }, () => new Response('due_at must be in the future', { status: 400 }));
		expect(await result).toMatchObject({ status: 400, data: { field: 'when', message: 'That time has already passed. Pick a later one.' } });
	});

	it('ticks off, cancels or snoozes a reminder, and nothing else', async () => {
		const snooze = run('reminder', { id: 'r1', do: 'snooze', minutes: '60' });
		expect(await snooze.result).toEqual({ reminder: 'snooze' });
		expect(snooze.api.calls[0].body).toEqual({ action: 'snooze', minutes: 60 });
		const done = run('reminder', { id: 'r1', do: 'done' });
		expect(await done.result).toEqual({ reminder: 'done' });
		expect(done.api.calls[0].body).toEqual({ action: 'done' });
		const odd = run('reminder', { id: 'r1', do: 'delete' });
		expect(await odd.result).toMatchObject({ status: 400 });
		expect(odd.api.calls).toEqual([]);
	});

	it('cancels scheduled work', async () => {
		const { api, result } = run('cancelTask', { id: 't1' });
		expect(await result).toEqual({ cancelledTask: 't1' });
		expect(api.calls[0].url).toBe('http://api.test/api/scheduled-tasks/t1/cancel');
	});

	it('passes on why the backend said no', async () => {
		const { result } = run('reminder', { id: 'gone', do: 'done' }, () => new Response('reminder not found', { status: 404 }));
		expect(await result).toMatchObject({ status: 404, data: { message: 'reminder not found' } });
	});

	it('saves both email settings', async () => {
		const { api, result } = run('preferences', { email_account: 'false', email_promos: 'true' }, () => json({ email_account: false, email_promos: true }));
		expect(await result).toEqual({ preferences: { email_account: false, email_promos: true } });
		expect(api.calls[0].body).toEqual({ email_account: false, email_promos: true });
	});
});
