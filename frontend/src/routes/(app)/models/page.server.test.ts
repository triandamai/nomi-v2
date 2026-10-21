import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions, load } = await import('./+page.server');

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });

const choice = { key: 'admin:a1', label: 'Sonnet', model_id: 'sonnet', provider_id: 'anthropic', provider_label: 'Anthropic', has_logo: true, chat_only: false };
const groups = [{ id: 'nomi:Anthropic', source: 'nomi', provider_id: 'anthropic', has_logo: true, label: 'Anthropic', connection_id: null, error: null, models: [] }];
const llmModels = { admin_models: [{ id: 'a1', label: 'Sonnet', provider: 'anthropic', model_id: 'sonnet' }], coding: { kind: 'default' }, default_coding_model_id: null };

type Answer = (path: string, init?: RequestInit) => Response | undefined;
function answer(override: Answer = () => undefined) {
	apiFetch.mockImplementation(async (_fetch: unknown, _cookies: unknown, path: string, init?: RequestInit) => {
		const custom = override(path, init);
		if (custom) return custom;
		if (path === '/api/llm/models') return json(llmModels);
		if (path === '/api/llm/model-options') return json({ groups, current: choice });
		return json({});
	});
}

const loadPage = () => load({ cookies: {} as Cookies, fetch } as unknown as Parameters<typeof load>[0]) as Promise<Record<string, unknown>>;

function event(fields: Record<string, string>) {
	const body = new FormData();
	for (const [name, value] of Object.entries(fields)) body.set(name, value);
	return {
		request: new Request('http://localhost/models?/x', { method: 'POST', body }),
		params: {},
		cookies: {} as Cookies,
		fetch,
	} as unknown as Parameters<NonNullable<typeof actions.setDefaultModel>>[0];
}

const calls = () => apiFetch.mock.calls.map(([, , path, init]) => `${(init as RequestInit | undefined)?.method ?? 'GET'} ${path}`);

beforeEach(() => {
	apiFetch.mockReset();
	answer();
});

describe('Model page: load', () => {
	it('loads the model options (the default) and the coding model settings', async () => {
		const data = await loadPage();
		expect(data.modelOptions).toEqual({ groups, current: choice });
		expect(data.models).toEqual(llmModels);
	});

	it("doesn't ask for providers or connections any more", async () => {
		await loadPage();
		expect(calls().sort()).toEqual(['GET /api/llm/model-options', 'GET /api/llm/models']);
	});

	it("says the model options couldn't be read rather than showing none", async () => {
		answer((path) => (path === '/api/llm/model-options' ? json({ error: 'boom' }, 500) : undefined));
		const data = await loadPage();
		expect(data.modelOptions).toBeNull();
		expect(data.models).toEqual(llmModels);
	});

	it('keeps the coding section usable when its settings fail', async () => {
		answer((path) => (path === '/api/llm/models' ? json({ error: 'boom' }, 500) : undefined));
		const data = await loadPage();
		expect(data.models).toEqual({ admin_models: [], coding: { kind: 'default' }, default_coding_model_id: null });
		expect(data.modelOptions).toEqual({ groups, current: choice });
	});
});

describe('Model page: actions', () => {
	it('setDefaultModel PUTs the key as the default for new chats', async () => {
		answer((path) => (path === '/api/llm/default-model' ? json({ model: choice }) : undefined));
		expect(await actions.setDefaultModel!(event({ key: 'admin:a1' }))).toEqual({ defaultModel: choice });
		expect(calls()).toEqual(['PUT /api/llm/default-model']);
		expect(JSON.parse(String((apiFetch.mock.calls[0][3] as RequestInit).body))).toEqual({ key: 'admin:a1' });
	});

	it('setDefaultModel refuses a malformed key and passes refusals on', async () => {
		expect(await actions.setDefaultModel!(event({ key: 'nope' }))).toMatchObject({ status: 400 });
		expect(apiFetch).not.toHaveBeenCalled();
		answer((path) => (path === '/api/llm/default-model' ? json({ error: 'not_found' }, 404) : undefined));
		expect(await actions.setDefaultModel!(event({ key: 'admin:a9' }))).toMatchObject({ status: 404 });
	});

	it('has Manage models and the coding model, and none of the old pickers', () => {
		expect(Object.keys(actions).sort()).toEqual(['connect', 'selectCodingModel', 'setDefaultModel', 'setModelVisibility']);
	});
});
