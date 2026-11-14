import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

function event(form?: Record<string, string>, reply: (call: BackendCall) => Response = () => json({ connections: [], providers: [] })) {
	const api = fakeBackend(reply);
	return { api, event: fakeEvent({ form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}

describe('providers', () => {
	it('lists the connected providers and the catalog to connect from', async () => {
		const { api, event: loading } = event(undefined, (call) => (call.url.endsWith('/connections') ? json({ connections: [{ id: 'c1' }] }) : json({ providers: [{ id: 'openai' }] })));
		expect(await load(loading)).toEqual({ connections: { ok: true, data: [{ id: 'c1' }] }, providers: [{ id: 'openai' }] });
		expect(api.calls.map((call) => call.url).sort()).toEqual(['http://api.test/api/admin/settings/llm/connections', 'http://api.test/api/llm/providers']);
	});

	it('connects one with its key, a label, an address and headers of its own', async () => {
		const { api, event: connecting } = event({ provider_id: 'custom', label: ' Lab GPU ', api_key: 'k', base_url: 'https://llm.lab.id/v1', headers: 'X-Team: nomi\n\nX-Region:  jkt ' }, () => json({ id: 'c2' }, 201));
		expect(await actions.connect(connecting)).toMatchObject({ action: 'connect', ok: true });
		expect(api.calls[0]).toMatchObject({
			method: 'POST',
			url: 'http://api.test/api/admin/settings/llm/connections',
			body: { provider_id: 'custom', label: 'Lab GPU', api_key: 'k', base_url: 'https://llm.lab.id/v1', headers: [{ name: 'X-Team', value: 'nomi' }, { name: 'X-Region', value: 'jkt' }] },
		});
	});

	it('refuses a header it can’t read, and a provider not chosen', async () => {
		for (const form of <Record<string, string>[]>[{ provider_id: '', api_key: 'k' }, { provider_id: 'custom', headers: 'no colon here' }]) {
			const { api, event: refused } = event(form);
			expect(await actions.connect(refused)).toMatchObject({ status: 400 });
			expect(api.calls).toEqual([]);
		}
	});

	it('changes a connection, an empty key keeping the saved one and an unchanged address not sent (sending one checks it live)', async () => {
		const { api, event: saving } = event({ id: 'c1', label: 'Main', api_key: '', base_url: 'https://llm.lab.id/v1', was_base_url: 'https://llm.lab.id/v1' }, () => json({ id: 'c1' }));
		expect(await actions.update(saving)).toMatchObject({ action: 'update', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/settings/llm/connections/c1' });
		expect(api.calls[0].body).toEqual({ label: 'Main' });

		const moved = event({ id: 'c1', label: 'Main', api_key: '', base_url: 'https://llm2.lab.id/v1', was_base_url: 'https://llm.lab.id/v1' }, () => json({ id: 'c1' }));
		await actions.update(moved.event);
		expect(moved.api.calls[0].body).toEqual({ label: 'Main', base_url: 'https://llm2.lab.id/v1' });
	});

	it('disconnects one, or says which models still use it', async () => {
		const gone = event({ id: 'c1' }, () => new Response(null, { status: 204 }));
		expect(await actions.disconnect(gone.event)).toMatchObject({ action: 'disconnect', ok: true });
		const used = event({ id: 'c1' }, () => json({ error: 'in_use', models: ['DeepSeek Flash'] }, 409));
		expect(await actions.disconnect(used.event)).toMatchObject({ status: 409, data: { problem: { code: 'in_use', models: ['DeepSeek Flash'] } } });
	});
});
