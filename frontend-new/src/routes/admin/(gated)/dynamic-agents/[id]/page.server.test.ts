import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const agent = { id: 'a1', name: 'Pak Tani', granted_tools: ['web_search'] };
const answer = (call: BackendCall) => (call.url.endsWith('/dynamic-agents') ? json([agent]) : json({ groups: [{ key: 'crew', kind: 'crew', label: 'Crew', tools: [{ name: 'web_search' }] }, { key: 'money', kind: 'agent', label: 'Dana', tools: [{ name: 'list_transactions' }] }, { key: 'custom', kind: 'custom', label: 'Custom agents', tools: [{ name: 'read_web_page' }] }] }));
const form = {
	name: ' Pak Tani ',
	intent_label: 'Farming',
	intent_description: 'Crops, seasons and fertiliser',
	system_prompt: 'You help small farmers.',
	tool_web_search: 'on',
	tool_read_web_page: 'on',
	supports_todos: 'on',
	supports_plans: '',
	can_delegate: '',
	shape: 'clover4',
	tone: 'citrus',
	motion: 'wobble',
};
function event(id: string, fields?: Record<string, string>, reply: (call: BackendCall) => Response = answer) {
	const api = fakeBackend(reply);
	return { api, event: fakeEvent({ params: { id }, form: fields, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}

describe('a custom agent', () => {
	it('opens new or from the list, with the tools it can be given', async () => {
		expect(await load(event('new').event)).toMatchObject({ agent: null, tools: ['read_web_page'] });
		expect(await load(event('a1').event)).toMatchObject({ title: 'Pak Tani', agent: { ok: true, data: agent } });
		await expect(load(event('nope').event)).rejects.toMatchObject({ status: 404 });
	});

	it('is created from the form and opens the list; saved in place when it exists', async () => {
		const created = event('new', form, () => json({ id: 'a9' }, 201));
		expect(await thrown(() => actions.save(created.event))).toMatchObject({ status: 303, location: '/admin/dynamic-agents' });
		expect(created.api.calls[0]).toMatchObject({
			method: 'POST',
			url: 'http://api.test/api/admin/dynamic-agents',
			body: {
				name: 'Pak Tani',
				intent_label: 'Farming',
				intent_description: 'Crops, seasons and fertiliser',
				system_prompt: 'You help small farmers.',
				granted_tools: ['read_web_page', 'web_search'],
				supports_todos: true,
				supports_plans: false,
				can_delegate: false,
				shape: 'clover4',
				tone: 'citrus',
				motion: 'wobble',
			},
		});
		const saved = event('a1', form, () => json(agent));
		expect(await actions.save(saved.event)).toMatchObject({ action: 'save', ok: true });
		expect(saved.api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/dynamic-agents/a1' });
	});

	it('refuses what the backend would, before sending', async () => {
		for (const wrong of [{ name: '' }, { system_prompt: ' ' }, { intent_label: '' }, { intent_description: '' }, { shape: 'star' }, { tone: 'neon' }, { motion: 'dance' }]) {
			const refused = event('a1', { ...form, ...wrong });
			expect(await actions.save(refused.event)).toMatchObject({ status: 400, data: { action: 'save' } });
			expect(refused.api.calls).toEqual([]);
		}
	});
});
