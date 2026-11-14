import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

describe('plans', () => {
	it('lists the plans with who is on each', async () => {
		const api = fakeBackend(() => json([{ id: 'p1', name: 'Free', subscribers: 12 }]));
		expect(await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toEqual({ plans: { ok: true, data: [{ id: 'p1', name: 'Free', subscribers: 12 }] } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/plans');
	});

	it('makes a plan the default for new people', async () => {
		const api = fakeBackend(() => new Response(null, { status: 204 }));
		const result = await actions.makeDefault(fakeEvent({ form: { id: 'p2' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(result).toMatchObject({ action: 'makeDefault', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/plans/p2/default' });
	});
});
