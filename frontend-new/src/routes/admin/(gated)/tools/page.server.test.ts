import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const speak = { name: 'speak', configurable: true, settings: {} };
function event(form?: Record<string, string>, reply: (call: BackendCall) => Response = () => json({})) {
	const api = fakeBackend(reply);
	return { api, event: fakeEvent({ form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}

describe('tools', () => {
	it('lists the tools by group, and the voices when the crew can speak', async () => {
		const { api, event: loading } = event(undefined, (call) => (call.url.endsWith('/tools') ? json({ groups: [{ key: 'crew', tools: [speak] }] }) : json({ voices: [{ voice_id: 'v1' }], models: [] })));
		expect(await load(loading)).toEqual({ tools: { ok: true, data: [{ key: 'crew', tools: [speak] }] }, voices: { voices: [{ voice_id: 'v1' }], models: [] } });
		expect(api.calls.map((call) => call.url)).toEqual(['http://api.test/api/admin/tools', 'http://api.test/api/admin/tools/speak/voices']);
	});

	it('shows the voice settings without a list of voices when the voice service can’t be reached', async () => {
		const { event: loading } = event(undefined, (call) => (call.url.endsWith('/tools') ? json({ groups: [{ key: 'crew', tools: [speak] }] }) : json({ error: 'no key' }, 400)));
		expect(await load(loading)).toMatchObject({ voices: null });
	});

	it('switches a tool on or off', async () => {
		const { api, event: switching } = event({ name: 'web_search', enabled: '' }, () => new Response(null, { status: 204 }));
		expect(await actions.toggle(switching)).toMatchObject({ action: 'toggle', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/tools/web_search', body: { enabled: false } });
	});

	it('saves web search: the provider, how many results, its address, a new key or forgetting it', async () => {
		const { api, event: saving } = event({ provider: 'searxng', max_results: '8', base_url: 'https://search.lab.id', api_key: '', clear_api_key: 'on' }, () => json({}));
		expect(await actions.saveSearch(saving)).toMatchObject({ action: 'saveSearch', ok: true });
		expect(api.calls[0]).toMatchObject({
			method: 'PUT',
			url: 'http://api.test/api/admin/tools/web_search/settings',
			body: { provider: 'searxng', max_results: 8, base_url: 'https://search.lab.id', api_key: null, clear_api_key: true },
		});
		const wrong = event({ provider: 'tavily', max_results: 'many' });
		expect(await actions.saveSearch(wrong.event)).toMatchObject({ status: 400 });
		expect(wrong.api.calls).toEqual([]);
	});

	it('saves how much of a page is read', async () => {
		const { api, event: saving } = event({ max_chars: '12 000' }, () => json({}));
		await actions.saveReadPage(saving);
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/tools/read_web_page/settings', body: { max_chars: 12000 } });
	});

	it('saves the voice, its tuning and its key', async () => {
		const { api, event: saving } = event(
			{ voice_id: 'v1', voice_name: 'Sarah', model_id: 'eleven_turbo', stability: '0.4', similarity_boost: '0.8', style: '0', use_speaker_boost: 'on', speed: '1.1', output_format: 'mp3_44100_128', max_chars: '2500', api_key: 'xi-new' },
			() => json({}),
		);
		expect(await actions.saveVoice(saving)).toMatchObject({ action: 'saveVoice', ok: true });
		expect(api.calls[0].body).toEqual({
			voice_id: 'v1',
			voice_name: 'Sarah',
			model_id: 'eleven_turbo',
			stability: 0.4,
			similarity_boost: 0.8,
			style: 0,
			use_speaker_boost: true,
			speed: 1.1,
			output_format: 'mp3_44100_128',
			max_chars: 2500,
			api_key: 'xi-new',
			clear_api_key: false,
		});
		const wrong = event({ voice_id: 'v1', model_id: 'm', stability: 'loud' });
		expect(await actions.saveVoice(wrong.event)).toMatchObject({ status: 400 });
	});

	it('tries web search with a question and shows what came back', async () => {
		const { api, event: testing } = event({ query: 'harga cabai hari ini' }, () => json({ ok: true, provider: 'tavily', results: [{ title: 'Harga', url: 'https://x.id', snippet: '…' }], error: null }));
		expect(await actions.testSearch(testing)).toMatchObject({ action: 'testSearch', ok: true, data: { ok: true, results: [{ title: 'Harga' }] } });
		expect(api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/tools/web_search/test', body: { query: 'harga cabai hari ini' } });
	});
});
