import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

describe('a billing term', () => {
	it('opens a new one without asking the backend', async () => {
		expect(await load(fakeEvent({ params: { id: 'new' }, locals: { accessToken: 't' } }) as never)).toMatchObject({ term: null });
	});

	it('finds an existing one in the list', async () => {
		const api = fakeBackend(() => json([{ id: 't1', months: 6, discount_percent: 10 }]));
		expect(await load(fakeEvent({ params: { id: 't1' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toMatchObject({ title: '6 months', term: { ok: true, data: { id: 't1' } } });
	});

	it('saves changes to a term', async () => {
		const api = fakeBackend(() => json({ id: 't1' }));
		const result = await actions.save(fakeEvent({ params: { id: 't1' }, form: { months: '6', discount_percent: '10', label: '', is_active: 'on', sort_order: '0' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(result).toMatchObject({ action: 'save', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/billing/terms/t1', body: { months: 6, discount_percent: 10, is_active: true } });
	});

	it('adds a term and goes back to the list', async () => {
		const api = fakeBackend(() => json({ id: 't2' }));
		await expect(actions.save(fakeEvent({ params: { id: 'new' }, form: { months: '12', discount_percent: '20' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never)).rejects.toMatchObject({ status: 303, location: '/admin/billing/terms' });
		expect(api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/billing/terms' });
	});

	it('refuses a term with no months before sending it', async () => {
		const api = fakeBackend(() => json({}));
		expect(await actions.save(fakeEvent({ params: { id: 'new' }, form: { months: '0' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toMatchObject({ status: 400 });
		expect(api.calls).toHaveLength(0);
	});
});
