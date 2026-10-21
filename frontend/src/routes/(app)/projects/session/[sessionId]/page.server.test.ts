import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions, load } = await import('./+page.server');

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });

const choice = { key: 'admin:a1', label: 'Sonnet', model_id: 'sonnet', provider_id: 'anthropic', provider_label: 'Anthropic', has_logo: true, chat_only: false };
const groups = [{ id: 'nomi:Anthropic', source: 'nomi', provider_id: 'anthropic', has_logo: true, label: 'Anthropic', connection_id: null, error: null, models: [] }];

type Answer = (path: string, init?: RequestInit) => Response | undefined;
function answer(override: Answer = () => undefined) {
	apiFetch.mockImplementation(async (_fetch: unknown, _cookies: unknown, path: string, init?: RequestInit) => {
		const custom = override(path, init);
		if (custom) return custom;
		if (path.startsWith('/api/sessions/s1/messages')) return json({ messages: [], has_more: false });
		if (path === '/api/sessions/s1/thinking') return json({ level: 'low' });
		if (path === '/api/llm/model-options') return json({ groups, current: choice });
		if (path === '/api/sessions/s1/model') return json({ model: { ...choice, chat_only: true } });
		if (path === '/api/projects/by-session/s1') return json({ error: 'not_found' }, 404);
		return json([]);
	});
}

const loadPage = () =>
	load({ params: { sessionId: 's1' }, cookies: {} as Cookies, fetch } as unknown as Parameters<typeof load>[0]) as Promise<Record<string, unknown>>;

beforeEach(() => {
	apiFetch.mockReset();
	answer();
});

describe('project chat: load', () => {
	it("loads the model picker: the person's options and this chat's model", async () => {
		const data = await loadPage();
		expect(data.modelOptions).toEqual({ groups, current: choice });
		expect(data.sessionModel).toEqual({ ...choice, chat_only: true });
	});

	it("still opens the project chat when the models can't be read", async () => {
		answer((path) => (path === '/api/llm/model-options' || path === '/api/sessions/s1/model' ? json({ error: 'boom' }, 500) : undefined));
		const data = await loadPage();
		expect(data.modelOptions).toEqual({ groups: [], current: null });
		expect(data.sessionModel).toBeNull();
		expect(data.messages).toEqual([]);
	});
});

describe('project chat: model actions', () => {
	const event = (fields: Record<string, string>) => {
		const body = new FormData();
		for (const [name, value] of Object.entries(fields)) body.set(name, value);
		return {
			request: new Request('http://localhost/projects/session/s1?/x', { method: 'POST', body }),
			params: { sessionId: 's1' },
			cookies: {} as Cookies,
			fetch,
		} as unknown as Parameters<NonNullable<typeof actions.setSessionModel>>[0];
	};

	it('offers the same model actions as a chat', () => {
		expect(actions.setSessionModel).toBeTypeOf('function');
		expect(actions.setModelVisibility).toBeTypeOf('function');
		expect(actions.connect).toBeTypeOf('function');
	});

	it("setSessionModel PUTs the project chat's model", async () => {
		const outcome = await actions.setSessionModel!(event({ key: 'conn:c1:openai/gpt-5' }));
		expect(outcome).toEqual({ sessionModel: { ...choice, chat_only: true } });
		const [, , path, init] = apiFetch.mock.calls[0];
		expect(path).toBe('/api/sessions/s1/model');
		expect((init as RequestInit).method).toBe('PUT');
		expect(JSON.parse(String((init as RequestInit).body))).toEqual({ key: 'conn:c1:openai/gpt-5' });
	});

	it('setModelVisibility saves the Manage models switches', async () => {
		const outcome = await actions.setModelVisibility!(event({ changes: JSON.stringify([{ key: 'admin:a1', enabled: true }]) }));
		expect(outcome).toEqual({ modelsSaved: true });
		expect(apiFetch.mock.calls[0][2]).toBe('/api/llm/model-options');
	});
});
