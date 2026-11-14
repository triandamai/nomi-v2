import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { act, field, getJson, mustLoad, send } = await import('./admin');

const event = (answer: Parameters<typeof fakeBackend>[0], form?: Record<string, string>) => {
	const api = fakeBackend(answer);
	return { api, event: fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' }, form }) };
};

describe('getJson', () => {
	it('reads what the backend answered, as the person', async () => {
		const { api, event: e } = event(() => json({ total_users: 3 }));
		expect(await getJson(e, '/api/admin/dashboard')).toEqual({ ok: true, data: { total_users: 3 } });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/admin/dashboard', method: 'GET', headers: { authorization: 'Bearer t' } });
	});

	it('keeps the backend’s own reason when it says no, and says so when it can’t be reached', async () => {
		expect(await getJson(event(() => new Response('failed to list users', { status: 500 })).event, '/x')).toEqual({ ok: false, problem: { status: 500, message: 'failed to list users' } });
		expect(await getJson(event(() => 'unreachable').event, '/x')).toEqual({ ok: false, problem: { status: 503, message: '' } });
	});

	it('turns a refused area into the console’s no-access page', async () => {
		const loaded = await getJson(event(() => new Response("you don't have access to this part of the admin console", { status: 403 })).event, '/x');
		expect(await thrown(() => mustLoad(loaded))).toMatchObject({ status: 403, body: { message: "you don't have access to this part of the admin console" } });
		expect(mustLoad({ ok: false, problem: { status: 500, message: 'x' } })).toEqual({ ok: false, problem: { status: 500, message: 'x' } });
	});
});

describe('send', () => {
	it('sends JSON and reads an empty answer as nothing', async () => {
		const { api, event: e } = event(() => new Response(null, { status: 204 }));
		expect(await send(e, '/api/admin/plans/p1/default', 'PUT')).toEqual({ ok: true, data: null });
		expect(await send(e, '/api/admin/plans', 'POST', { name: 'Pro' })).toEqual({ ok: true, data: null });
		expect(api.calls[1]).toMatchObject({ method: 'POST', body: { name: 'Pro' } });
	});
});

describe('act', () => {
	it('answers a form with what happened, named after the action', async () => {
		const { event: e } = event(() => json({ id: 'b1', recipients: 4 }, 201));
		expect(await act(e, 'broadcast', '/api/admin/notifications/broadcast', 'POST', { title: 'Hi' })).toEqual({ action: 'broadcast', ok: true, data: { id: 'b1', recipients: 4 } });
	});

	it('fails the form with the backend’s status and reason', async () => {
		const { event: e } = event(() => new Response('make another plan the default first', { status: 400 }));
		expect(await act(e, 'delete', '/api/admin/plans/p1', 'DELETE')).toMatchObject({
			status: 400,
			data: { action: 'delete', ok: false, problem: { status: 400, message: 'make another plan the default first' } },
		});
	});
});

describe('field', () => {
	it('reads a form field, trimmed, empty when missing', async () => {
		const e = fakeEvent({ form: { name: '  Pro ' } });
		const form = await e.request.formData();
		expect(field(form, 'name')).toBe('Pro');
		expect(field(form, 'missing')).toBe('');
	});
});
