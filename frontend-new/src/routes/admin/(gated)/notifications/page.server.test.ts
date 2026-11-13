import { describe, expect, it, vi } from 'vitest';
import type { Access } from '$lib/admin/access';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

function event(access: Access, options: { form?: Record<string, string>; answer?: (call: BackendCall) => Response } = {}) {
	const api = fakeBackend(options.answer ?? (() => json([])));
	const base = fakeEvent({ form: options.form, fetch: api.fetch, locals: { accessToken: 't' } });
	return { api, event: { ...base, parent: async () => ({ access }) } as never };
}

describe('notifications', () => {
	it('lists what was sent, and the plans to send to when the viewer may see them', async () => {
		const quiet = event({ notifications: 'view' });
		expect(await load(quiet.event)).toEqual({ broadcasts: { ok: true, data: [] }, plans: null });
		expect(quiet.api.calls.map((call) => call.url)).toEqual(['http://api.test/api/admin/notifications/broadcasts']);

		const both = event({ notifications: 'write', plans: 'view' });
		await load(both.event);
		expect(both.api.calls.map((call) => call.url).sort()).toEqual(['http://api.test/api/admin/notifications/broadcasts', 'http://api.test/api/admin/plans']);
	});

	it('sends to everyone or one plan, a link only to a page in the app', async () => {
		const sent = event({ notifications: 'write' }, { form: { title: ' New voices ', body: 'Kala can read reminders aloud.', link: '/settings', audience: 'all' }, answer: () => json({ id: 'b1', recipients: 40 }, 201) });
		expect(await actions.send(sent.event)).toMatchObject({ action: 'send', ok: true, data: { recipients: 40 } });
		expect(sent.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/notifications/broadcast', body: { title: 'New voices', body: 'Kala can read reminders aloud.', link: '/settings', audience: 'all' } });

		const toPlan = event({ notifications: 'write' }, { form: { title: 'Hi', body: 'Pro news', link: '', audience: 'p2' }, answer: () => json({ id: 'b2', recipients: 3 }, 201) });
		await actions.send(toPlan.event);
		expect(toPlan.api.calls[0].body).toEqual({ title: 'Hi', body: 'Pro news', link: null, audience: 'p2' });

		for (const wrong of [{ title: '' }, { body: ' ' }, { link: 'https://example.com' }]) {
			const refused = event({ notifications: 'write' }, { form: { title: 'Hi', body: 'There', link: '', audience: 'all', ...wrong } });
			expect(await actions.send(refused.event)).toMatchObject({ status: 400, data: { action: 'send' } });
			expect(refused.api.calls).toEqual([]);
		}
	});
});
