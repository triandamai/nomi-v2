import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

function event(form?: Record<string, string>, reply: (call: BackendCall) => Response = () => json({ provider: 'openai', model_id: 'text-embedding-3-small', base_url: null, api_key_masked: '...abcd' })) {
	const api = fakeBackend(reply);
	return { api, event: fakeEvent({ form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}

describe('embedding', () => {
	it('reads the embedding model', async () => {
		const { api, event: loading } = event();
		expect(await load(loading)).toMatchObject({ embedding: { ok: true, data: { provider: 'openai' } } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/settings/embedding');
	});

	it('says none is saved here when the backend has none (Nomi then uses its environment’s)', async () => {
		const { event: loading } = event(undefined, () => new Response('settings not configured', { status: 404 }));
		expect(await load(loading)).toEqual({ embedding: { ok: true, data: null } });
	});

	it('saves it, an empty key keeping the saved one', async () => {
		const { api, event: saving } = event({ provider: 'gemini', model_id: ' text-embedding-004 ', api_key: '', base_url: '' });
		expect(await actions.save(saving)).toMatchObject({ action: 'save', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/settings/embedding', body: { provider: 'gemini', model_id: 'text-embedding-004', api_key: null, base_url: null } });
	});

	it('refuses a provider the backend doesn’t know, and an empty model', async () => {
		for (const form of [{ provider: 'voyage', model_id: 'v3' }, { provider: 'openai', model_id: '' }]) {
			const { api, event: refused } = event(form);
			expect(await actions.save(refused)).toMatchObject({ status: 400 });
			expect(api.calls).toEqual([]);
		}
	});

	it('finds the provider’s embedding models', async () => {
		const { api, event: finding } = event({ provider: 'openai', api_key: '', base_url: '' }, () => json({ models: [{ id: 'text-embedding-3-large' }] }));
		expect(await actions.findModels(finding)).toMatchObject({ action: 'findModels', ok: true, data: { models: [{ id: 'text-embedding-3-large' }] } });
		expect(api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/settings/embedding/fetch-models', body: { provider: 'openai', api_key: null, base_url: null } });
	});
});
