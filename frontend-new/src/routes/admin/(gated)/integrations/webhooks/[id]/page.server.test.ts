import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

describe('an endpoint’s deliveries', () => {
	it('lists them, narrowed to a status, with the endpoint they went to', async () => {
		const api = fakeBackend((call: BackendCall) => (call.url.includes('/deliveries') ? json([{ id: 'd1' }]) : json([{ id: 'w1', url: 'https://x.id' }, { id: 'w2' }])));
		const data = await load(fakeEvent({ url: 'http://localhost/admin/integrations/webhooks/w1?status=failed', params: { id: 'w1' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(data).toMatchObject({ title: 'https://x.id', status: 'failed', endpoint: { id: 'w1' }, deliveries: { ok: true, data: [{ id: 'd1' }] } });
		expect(api.calls.map((call) => call.url).sort()).toEqual(['http://api.test/api/admin/shipyard/webhooks', 'http://api.test/api/admin/shipyard/webhooks/w1/deliveries?status=failed']);
	});

	it('sends one again', async () => {
		const api = fakeBackend(() => json({}));
		expect(await actions.redeliver(fakeEvent({ params: { id: 'w1' }, form: { id: 'd1' }, fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toMatchObject({ action: 'redeliver', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/shipyard/deliveries/d1/redeliver' });
	});
});
