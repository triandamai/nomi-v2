import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

describe('coupons', () => {
	it('lists the coupons matching the code, with the plans by name', async () => {
		const api = fakeBackend((call) => (call.url.endsWith('/coupons') ? json([{ id: 'c1', code: 'LEBARAN25' }, { id: 'c2', code: 'HEMAT' }]) : json([{ id: 'p1', name: 'Pro', slug: 'pro' }])));
		const loaded = await load(fakeEvent({ url: 'http://localhost/admin/billing/coupons?query=leb', fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(loaded).toEqual({ query: 'leb', coupons: { ok: true, data: [{ id: 'c1', code: 'LEBARAN25' }] }, total: 2, plans: [{ id: 'p1', name: 'Pro' }] });
	});
});
