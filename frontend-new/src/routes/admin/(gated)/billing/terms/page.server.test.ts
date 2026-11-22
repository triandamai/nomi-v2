import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

describe('billing terms', () => {
	it('lists the terms in their order', async () => {
		const api = fakeBackend(() => json([{ id: 'b', months: 12, sort_order: 1 }, { id: 'a', months: 6, sort_order: 1 }, { id: 'c', months: 1, sort_order: 0 }]));
		const loaded = (await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } }) as never)) as { terms: { ok: boolean; data: { id: string }[] } };
		expect(loaded.terms.data.map((term) => term.id)).toEqual(['c', 'a', 'b']);
		expect(api.calls[0].url).toBe('http://api.test/api/admin/billing/terms');
	});
});
