import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

describe('custom agents', () => {
	it('lists them', async () => {
		const api = fakeBackend(() => json([{ id: 'a1', name: 'Pak Tani' }]));
		expect(await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toEqual({ agents: { ok: true, data: [{ id: 'a1', name: 'Pak Tani' }] } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/dynamic-agents');
	});

	it('switches one on or off', async () => {
		const api = fakeBackend(() => json({ id: 'a1', is_active: false }));
		expect(await actions.toggle(fakeEvent({ form: { id: 'a1' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toMatchObject({ action: 'toggle', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/dynamic-agents/a1/toggle-active' });
	});
});
