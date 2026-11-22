import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

describe('agent switches', () => {
	it('lists the built-in crew', async () => {
		const api = fakeBackend(() => json({ agents: [{ agent_type: 'coding', name: 'Koda', role: 'Builds apps', enabled: true, locked: false }] }));
		expect(await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toEqual({ crew: { ok: true, data: [{ agent_type: 'coding', name: 'Koda', role: 'Builds apps', enabled: true, locked: false }] } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/crew');
	});

	it('switches a member off for everyone', async () => {
		const api = fakeBackend(() => new Response(null, { status: 204 }));
		expect(await actions.toggle(fakeEvent({ form: { agent_type: 'coding', enabled: 'false' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toMatchObject({ action: 'toggle', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/crew/coding', body: { enabled: false } });
	});

	it('refuses without a member', async () => {
		expect(await actions.toggle(fakeEvent({ form: {}, locals: { accessToken: 't' } }) as never)).toMatchObject({ status: 400 });
	});
});
