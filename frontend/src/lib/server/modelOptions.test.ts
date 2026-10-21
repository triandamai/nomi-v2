import { beforeEach, describe, expect, it, vi } from 'vitest';
import { isRedirect, redirect, type Cookies } from '@sveltejs/kit';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { fetchModelOptions, loadModelOptions, loadSessionModel, manageModelActions, saveDefaultModel, saveModelVisibility, saveSessionModel, sessionModelActions } = await import(
	'./modelOptions'
);

const cookies = {} as Cookies;
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
const form = (fields: Record<string, string>) => {
	const data = new FormData();
	for (const [name, value] of Object.entries(fields)) data.set(name, value);
	return data;
};

const choice = { key: 'admin:a1', label: 'Sonnet', model_id: 'sonnet', provider_id: 'anthropic', provider_label: 'Anthropic', has_logo: true, chat_only: true };

beforeEach(() => apiFetch.mockReset());

describe('loadModelOptions', () => {
	it('reads the groups and the default model', async () => {
		apiFetch.mockResolvedValue(json({ groups: [{ id: 'nomi:x' }], current: choice }));
		expect(await loadModelOptions(fetch, cookies)).toEqual({ groups: [{ id: 'nomi:x' }], current: choice });
		expect(apiFetch.mock.calls[0][2]).toBe('/api/llm/model-options');
	});

	it('falls back to nothing when the API fails, never breaking the chat', async () => {
		apiFetch.mockResolvedValue(json({ error: 'nope' }, 500));
		expect(await loadModelOptions(fetch, cookies)).toEqual({ groups: [], current: null });
		apiFetch.mockRejectedValue(new Error('down'));
		expect(await loadModelOptions(fetch, cookies)).toEqual({ groups: [], current: null });
		apiFetch.mockResolvedValue(json({ something: 'else' }));
		expect(await loadModelOptions(fetch, cookies)).toEqual({ groups: [], current: null });
	});

	it('still sends a signed-out person to log in', async () => {
		let signIn: unknown;
		try {
			redirect(303, '/login');
		} catch (e) {
			signIn = e;
		}
		apiFetch.mockRejectedValueOnce(signIn);
		const thrown = await loadModelOptions(fetch, cookies).then(
			() => null,
			(e: unknown) => e,
		);
		expect(isRedirect(thrown) && thrown.location).toBe('/login');
	});
});

describe('loadSessionModel', () => {
	it("reads the chat's model", async () => {
		apiFetch.mockResolvedValue(json({ model: choice }));
		expect(await loadSessionModel(fetch, cookies, 's1')).toEqual(choice);
		expect(apiFetch.mock.calls[0][2]).toBe('/api/sessions/s1/model');
	});

	it('is null when it can’t be read', async () => {
		apiFetch.mockResolvedValue(json({ error: 'not_found' }, 404));
		expect(await loadSessionModel(fetch, cookies, 's1')).toBeNull();
	});
});

describe('saveSessionModel', () => {
	it('PUTs the key and returns the model the chat now uses', async () => {
		apiFetch.mockResolvedValue(json({ model: choice }));
		expect(await saveSessionModel(fetch, cookies, 's1', form({ key: 'admin:a1' }))).toEqual({ sessionModel: choice });
		const [, , path, init] = apiFetch.mock.calls[0];
		expect(path).toBe('/api/sessions/s1/model');
		expect(init.method).toBe('PUT');
		expect(JSON.parse(init.body)).toEqual({ key: 'admin:a1' });
	});

	it('refuses a missing or malformed key without asking the API', async () => {
		const result = await saveSessionModel(fetch, cookies, 's1', form({ key: 'nope' }));
		expect(result).toMatchObject({ status: 400, data: { modelError: expect.any(String) } });
		expect(apiFetch).not.toHaveBeenCalled();
	});

	it("passes the API's refusal on", async () => {
		apiFetch.mockResolvedValue(json({ error: 'forbidden' }, 403));
		expect(await saveSessionModel(fetch, cookies, 's1', form({ key: 'conn:c1:gpt' }))).toMatchObject({ status: 403, data: { modelError: expect.any(String) } });
	});
});

describe('saveModelVisibility', () => {
	it('PUTs the changes', async () => {
		apiFetch.mockResolvedValue(new Response(null, { status: 204 }));
		const changes = [{ key: 'admin:a1', enabled: false }, { key: 'conn:c1:x/y:free', enabled: true }];
		expect(await saveModelVisibility(fetch, cookies, form({ changes: JSON.stringify(changes) }))).toEqual({ modelsSaved: true });
		const [, , path, init] = apiFetch.mock.calls[0];
		expect(path).toBe('/api/llm/model-options');
		expect(init.method).toBe('PUT');
		expect(JSON.parse(init.body)).toEqual({ changes });
	});

	it('refuses changes that are not a list of keys and on/off', async () => {
		for (const changes of ['', 'nope', '{}', '[]', JSON.stringify([{ key: 'admin:a1', enabled: 'yes' }]), JSON.stringify([{ key: 'bad', enabled: true }])]) {
			expect(await saveModelVisibility(fetch, cookies, form({ changes }))).toMatchObject({ status: 400 });
		}
		expect(apiFetch).not.toHaveBeenCalled();
	});

	it("passes the API's refusal on", async () => {
		apiFetch.mockResolvedValue(json({ error: 'invalid_key' }, 400));
		const result = await saveModelVisibility(fetch, cookies, form({ changes: JSON.stringify([{ key: 'admin:a1', enabled: true }]) }));
		expect(result).toMatchObject({ status: 400, data: { modelError: expect.any(String) } });
	});
});

describe('fetchModelOptions', () => {
	it('reads the groups and the default model', async () => {
		apiFetch.mockResolvedValue(json({ groups: [], current: choice }));
		expect(await fetchModelOptions(fetch, cookies)).toEqual({ groups: [], current: choice });
	});

	it('is null when they can’t be read, so the Model page can say so', async () => {
		apiFetch.mockResolvedValue(json({ error: 'nope' }, 500));
		expect(await fetchModelOptions(fetch, cookies)).toBeNull();
		apiFetch.mockRejectedValue(new Error('down'));
		expect(await fetchModelOptions(fetch, cookies)).toBeNull();
		apiFetch.mockResolvedValue(json({ something: 'else' }));
		expect(await fetchModelOptions(fetch, cookies)).toBeNull();
	});
});

describe('saveDefaultModel', () => {
	const fallback = { ...choice, chat_only: false };

	it('PUTs the key and returns the new default', async () => {
		apiFetch.mockResolvedValue(json({ model: fallback }));
		expect(await saveDefaultModel(fetch, cookies, form({ key: 'conn:c1:openai/gpt-5' }))).toEqual({ defaultModel: fallback });
		const [, , path, init] = apiFetch.mock.calls[0];
		expect(path).toBe('/api/llm/default-model');
		expect(init.method).toBe('PUT');
		expect(JSON.parse(init.body)).toEqual({ key: 'conn:c1:openai/gpt-5' });
	});

	it('refuses a missing or malformed key without asking the API', async () => {
		for (const fields of [{}, { key: '' }, { key: 'admin:' }, { key: 'conn:c1' }, { key: 'gpt-5' }] as Record<string, string>[]) {
			expect(await saveDefaultModel(fetch, cookies, form(fields))).toMatchObject({ status: 400, data: { modelError: expect.any(String) } });
		}
		expect(apiFetch).not.toHaveBeenCalled();
	});

	it("passes the API's refusal on (a switched-off model, someone else's provider)", async () => {
		apiFetch.mockResolvedValue(json({ error: 'not_found' }, 404));
		expect(await saveDefaultModel(fetch, cookies, form({ key: 'admin:a9' }))).toMatchObject({ status: 404, data: { modelError: expect.any(String) } });
	});
});

describe('shared model picker actions', () => {
	const event = <P extends Record<string, string>>(fields: Record<string, string>, params = {} as P) => ({
		request: new Request('http://localhost/x', { method: 'POST', body: form(fields) }),
		cookies,
		fetch,
		params,
	});

	it("setSessionModel sets the model of the page's chat", async () => {
		apiFetch.mockResolvedValue(json({ model: choice }));
		expect(await sessionModelActions.setSessionModel(event({ key: 'admin:a1' }, { sessionId: 's9' }))).toEqual({ sessionModel: choice });
		const [, , path, init] = apiFetch.mock.calls[0];
		expect(path).toBe('/api/sessions/s9/model');
		expect(JSON.parse(init.body)).toEqual({ key: 'admin:a1' });
	});

	it('setModelVisibility saves the switches', async () => {
		apiFetch.mockResolvedValue(new Response(null, { status: 204 }));
		const changes = [{ key: 'admin:a1', enabled: false }];
		expect(await manageModelActions.setModelVisibility(event({ changes: JSON.stringify(changes) }))).toEqual({ modelsSaved: true });
		expect(apiFetch.mock.calls[0][2]).toBe('/api/llm/model-options');
	});

	it("connect adds one of the person's own providers", async () => {
		apiFetch.mockResolvedValue(json({ id: 'c2', provider_id: 'openai' }, 201));
		const outcome = await manageModelActions.connect(event({ provider_id: 'openai', api_key: 'sk-test' }));
		expect(outcome).toEqual({ connection: { id: 'c2', provider_id: 'openai' } });
		const [, , path, init] = apiFetch.mock.calls[0];
		expect(path).toBe('/api/llm/connections');
		expect(init.method).toBe('POST');
	});

	it('the session bundle carries Manage models too', () => {
		expect(Object.keys(sessionModelActions).sort()).toEqual(['connect', 'setModelVisibility', 'setSessionModel']);
		expect(Object.keys(manageModelActions).sort()).toEqual(['connect', 'setModelVisibility']);
	});
});
