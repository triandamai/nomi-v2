import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json } from '../../../../test-stubs/server';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const agentData = {
	id: 'a1',
	name: 'Researcher',
	system_prompt: 'Search and analyze',
	intent_label: 'Research',
	intent_description: 'Deep dive into topics',
	granted_tools: ['web_search'],
	supports_todos: true,
	supports_plans: true,
	can_delegate: false,
	shape: 'cookie9',
	tone: 'glow',
	motion: 'spin',
	is_active: true,
};

describe('dynamic-agents page.server', () => {
	it('load fetches dynamic agents list', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		vi.stubGlobal('fetch', vi.fn(async () => json([agentData])));

		const data = await load(event({ cookies }));
		expect(data).toMatchObject({ agents: [agentData] });
	});

	it('create validates fields and posts to API', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		const bad = await actions.create(event({ cookies, form: { name: '' } }));
		expect(bad).toMatchObject({ status: 400 });

		vi.stubGlobal('fetch', vi.fn(async () => json({ success: true })));
		const ok = await actions.create(
			event({
				cookies,
				form: {
					name: 'Researcher',
					system_prompt: 'Prompt',
					intent_label: 'Label',
					intent_description: 'Desc',
					granted_tools: ['tool1'],
				},
			}),
		);
		expect(ok).toMatchObject({ success: true });
	});

	it('update validates fields and puts to API', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		const bad = await actions.update(event({ cookies, form: { id: 'a1', name: '' } }));
		expect(bad).toMatchObject({ status: 400 });

		vi.stubGlobal('fetch', vi.fn(async () => json({ success: true })));
		const ok = await actions.update(
			event({
				cookies,
				form: {
					id: 'a1',
					name: 'Researcher',
					system_prompt: 'Prompt',
					intent_label: 'Label',
					intent_description: 'Desc',
				},
			}),
		);
		expect(ok).toMatchObject({ success: true });
	});

	it('toggleActive toggles dynamic agent state', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		const bad = await actions.toggleActive(event({ cookies, form: {} }));
		expect(bad).toMatchObject({ status: 400 });

		vi.stubGlobal('fetch', vi.fn(async () => json({ success: true })));
		const ok = await actions.toggleActive(event({ cookies, form: { id: 'a1' } }));
		expect(ok).toMatchObject({ success: true });
	});
});
