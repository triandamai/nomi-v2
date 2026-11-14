import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

describe('events received', () => {
	it('asks for the page in the address, the first without one', async () => {
		for (const [url, asked] of [
			['http://localhost/admin/integrations/events?page=3', 'http://api.test/api/admin/shipyard/events?page=3'],
			['http://localhost/admin/integrations/events', 'http://api.test/api/admin/shipyard/events?page=1'],
			['http://localhost/admin/integrations/events?page=x', 'http://api.test/api/admin/shipyard/events?page=1'],
		]) {
			const api = fakeBackend(() => json({ events: [], page: 1, pages: 1, total: 0 }));
			await load(fakeEvent({ url, fetch: api.fetch, locals: { accessToken: 't' } }) as never);
			expect(api.calls[0].url).toBe(asked);
		}
	});
});
