import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json } from '../../../../test-stubs/server';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const toolGroups = [
	{
		key: 'web',
		kind: 'agent' as const,
		label: 'Web Tools',
		tools: [
			{
				name: 'web_search',
				description: 'Search the web',
				enabled: true,
				required: false,
				agent_only: false,
				configurable: true,
				ready: true,
				used_by: [],
				settings: null,
			},
		],
	},
];

describe('tools page.server', () => {
	it('load fetches tool groups', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		vi.stubGlobal('fetch', vi.fn(async () => json({ groups: toolGroups })));

		const data = await load(event({ cookies }));
		expect(data).toMatchObject({ groups: toolGroups });
	});

	it('toggle switches tool active state', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		const bad = await actions.toggle(event({ cookies, form: {} }));
		expect(bad).toMatchObject({ status: 400 });

		vi.stubGlobal('fetch', vi.fn(async () => json({ ok: true })));
		const ok = await actions.toggle(event({ cookies, form: { name: 'web_search', enabled: 'true' } }));
		expect(ok).toMatchObject({ success: true });
	});

	it('saveSearch, saveReadPage, saveVoice save tool configs', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		vi.stubGlobal('fetch', vi.fn(async () => json({ ok: true })));

		const s1 = await actions.saveSearch(event({ cookies, form: { provider: 'tavily', max_results: '5' } }));
		expect(s1).toMatchObject({ saved: 'web_search' });

		const s2 = await actions.saveReadPage(event({ cookies, form: { max_chars: '10000' } }));
		expect(s2).toMatchObject({ saved: 'read_web_page' });

		const s3 = await actions.saveVoice(event({ cookies, form: { voice_id: 'v1', model_id: 'm1' } }));
		expect(s3).toMatchObject({ saved: 'speak' });
	});

	it('test runs a search test query', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				json({
					ok: true,
					provider: 'tavily',
					results: [{ title: 'Example', url: 'https://example.com', snippet: 'A snippet' }],
					error: null,
				}),
			),
		);

		const res = await actions.test(event({ cookies, form: { query: 'vitest test' } }));
		expect(res).toMatchObject({ test: { ok: true, provider: 'tavily' } });
	});
});
