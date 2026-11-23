import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

describe('a project’s issues', () => {
	it('reads them, titled with the project', async () => {
		const api = fakeBackend(() => json({ project: { id: 'p1', name: 'warung-kopi-menu' }, issues: [{ number: 1, title: 'Menu tidak muncul', status: 'open' }] }));
		const loaded = await load(fakeEvent({ params: { id: 'p1' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(loaded).toMatchObject({ title: 'warung-kopi-menu · issues', issues: { ok: true } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/projects/p1/issues');
	});
});
