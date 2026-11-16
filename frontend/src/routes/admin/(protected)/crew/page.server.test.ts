import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json } from '../../../../test-stubs/server';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const crew = {
	agents: [
		{ agent_type: 'chitchat', name: 'Nomi', role: 'Talks with you', enabled: true, locked: true },
		{ agent_type: 'design', name: 'Sora', role: 'Designs sites and apps', enabled: true, locked: false },
	],
};

describe('admin crew page.server', () => {
	it('load lists every built-in agent and whether it is on', async () => {
		const fetchMock = vi.fn(async (_url: string) => json(crew));
		vi.stubGlobal('fetch', fetchMock);
		const data = await load(event({ cookies: cookieJar({ access_token: 'valid' }) }));
		expect(data).toEqual(crew);
		expect(String(fetchMock.mock.calls[0][0])).toContain('/api/admin/crew');
	});

	it('load copes with a backend that cannot list them', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({}, 500)));
		expect(await load(event({ cookies: cookieJar({ access_token: 'valid' }) }))).toEqual({ agents: [] });
	});

	it('toggle switches one agent on or off for everyone', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		const fetchMock = vi.fn(async (_url: string, _init?: RequestInit) => json(crew));
		vi.stubGlobal('fetch', fetchMock);
		const ok = await actions.toggle(event({ cookies, form: { agent_type: 'design', enabled: 'false' } }));
		expect(ok).toMatchObject({ success: true });
		const [url, init] = fetchMock.mock.calls[0];
		expect(String(url)).toContain('/api/admin/crew/design');
		expect(init?.method).toBe('PUT');
		expect(JSON.parse(String(init?.body))).toEqual({ enabled: false });
	});

	it('toggle refuses a form without an agent, and reports what the backend refused', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		expect(await actions.toggle(event({ cookies, form: { enabled: 'true' } }))).toMatchObject({ status: 400 });
		vi.stubGlobal('fetch', vi.fn(async () => json({}, 409)));
		expect(await actions.toggle(event({ cookies, form: { agent_type: 'chitchat', enabled: 'false' } }))).toMatchObject({ status: 409 });
	});
});
