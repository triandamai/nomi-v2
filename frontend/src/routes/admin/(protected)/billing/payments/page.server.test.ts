import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json } from '../../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { load } = await import('./+page.server');

beforeEach(() => {
	apiFetch.mockReset();
	apiFetch.mockImplementation(async () => json({ items: [{ id: 'p1' }], total: 1, page: 2, page_size: 20 }));
});

describe('admin payments: load', () => {
	it('passes the page and a known status to the backend', async () => {
		const data = await load(event({ url: 'http://localhost/admin/billing/payments?status=paid&page=2' }) as never);
		expect(apiFetch.mock.calls[0][2]).toBe('/api/admin/payments?page=2&status=paid');
		expect(data).toMatchObject({ items: [{ id: 'p1' }], status: 'paid', total: 1 });
	});

	it('drops a status the backend does not know and a page that is not a number', async () => {
		await load(event({ url: 'http://localhost/admin/billing/payments?status=bogus&page=x' }) as never);
		expect(apiFetch.mock.calls[0][2]).toBe('/api/admin/payments?page=1');
	});
});
