import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

describe('admin projects', () => {
	it('lists every project, searched by name or owner', async () => {
		const api = fakeBackend(() => json({ projects: [{ id: 'p1', name: 'warung-kopi-menu' }] }));
		const loaded = await load(fakeEvent({ url: 'http://localhost/admin/projects?query=kopi', fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(loaded).toEqual({ query: 'kopi', projects: { ok: true, data: [{ id: 'p1', name: 'warung-kopi-menu' }] } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/projects?q=kopi');
	});

	it('says why the list failed', async () => {
		const api = fakeBackend(() => new Response('database down', { status: 500 }));
		expect(await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toMatchObject({ projects: { ok: false, problem: { status: 500 } } });
	});
});
