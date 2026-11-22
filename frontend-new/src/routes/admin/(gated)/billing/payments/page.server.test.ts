import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

describe('payments', () => {
	it('reads a page of one state', async () => {
		const api = fakeBackend(() => json({ items: [], total: 0, page: 2, page_size: 20 }));
		expect(await load(fakeEvent({ url: 'http://localhost/admin/billing/payments?status=paid&page=2', fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toMatchObject({ status: 'paid', page: 2, payments: { ok: true } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/payments?page=2&status=paid');
	});

	it('drops a state it doesn’t know', async () => {
		const api = fakeBackend(() => json({ items: [], total: 0, page: 1, page_size: 20 }));
		await load(fakeEvent({ url: 'http://localhost/admin/billing/payments?status=refunded', fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(api.calls[0].url).toBe('http://api.test/api/admin/payments?page=1');
	});
});
