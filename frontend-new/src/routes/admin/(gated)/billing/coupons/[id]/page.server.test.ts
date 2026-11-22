import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { actions } = await import('./+page.server');

describe('a coupon', () => {
	it('saves a coupon limited to some plans', async () => {
		const api = fakeBackend(() => json({ id: 'c1' }));
		const result = await actions.save(fakeEvent({ params: { id: 'c1' }, form: { code: 'lebaran25', discount_kind: 'percent', discount_value: '25', is_active: 'on' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(result).toMatchObject({ action: 'save', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/coupons/c1', body: { code: 'LEBARAN25', discount_value: 25, is_active: true } });
	});

	it('shows the backend’s words when a used coupon can’t be deleted', async () => {
		const api = fakeBackend(() => new Response('coupon has been used; deactivate it instead', { status: 409 }));
		const result = await actions.delete(fakeEvent({ params: { id: 'c1' }, form: {}, fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(result).toMatchObject({ status: 409, data: { action: 'delete', ok: false, problem: { message: 'coupon has been used; deactivate it instead' } } });
	});
});
