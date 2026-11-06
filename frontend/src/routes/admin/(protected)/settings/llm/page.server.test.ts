import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json, textResponse } from '../../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions, load } = await import('./+page.server');

const calls = () => apiFetch.mock.calls.map(([, , path, init]) => ({ path, method: init?.method ?? 'GET', body: init?.body ? JSON.parse(init.body) : undefined }));
const status = (result: unknown) => (result as { status?: number }).status;

beforeEach(() => {
	apiFetch.mockReset();
	apiFetch.mockImplementation(async () => json({ models: [{ id: 'm', label: null }] }));
});

describe('admin models: load', () => {
	it('loads the models, shared connections and catalog, empty when they fail', async () => {
		apiFetch.mockImplementation(async (_f: unknown, _c: unknown, path: string) => (path === '/api/admin/settings/llm/models' ? json([{ id: 'a1' }]) : json({}, 500)));
		expect(await load(event())).toMatchObject({ models: [{ id: 'a1' }], connections: [], providers: [] });
		apiFetch.mockImplementation(async () => json({}, 500));
		expect(await load(event())).toMatchObject({ models: [] });
	});
});

describe('admin models: create and update', () => {
	it('needs a label and a provider or connection, and prices that are numbers', async () => {
		expect(status(await actions.create(event({ form: { label: '', provider: 'anthropic' } })))).toBe(400);
		expect(status(await actions.create(event({ form: { label: 'Sonnet' } })))).toBe(400);
		expect(status(await actions.create(event({ form: { label: 'Sonnet', provider: 'anthropic', input_usd_per_mtok: 'cheap' } })))).toBe(400);
		expect(status(await actions.create(event({ form: { label: 'Sonnet', provider: 'anthropic', output_usd_per_mtok: '-1' } })))).toBe(400);
		expect(status(await actions.update(event({ form: { label: 'Sonnet', provider: 'anthropic' } })))).toBe(400);
		expect(status(await actions.update(event({ form: { id: 'a1', label: 'S', provider: 'x', cached_input_usd_per_mtok: 'NaN' } })))).toBe(400);
		expect(apiFetch).not.toHaveBeenCalled();
	});

	it('sends a model on a shared connection without a key of its own', async () => {
		expect(await actions.create(event({ form: { label: 'Sonnet', connection_id: 'c1', model_id: 'sonnet', input_usd_per_mtok: '3', output_usd_per_mtok: ' ', media_mode: 'listed', media: ['image', 'pdf'] } }))).toEqual({ success: true });
		expect(calls()[0]).toEqual({
			path: '/api/admin/settings/llm/models',
			method: 'POST',
			body: { label: 'Sonnet', connection_id: 'c1', model_id: 'sonnet', input_usd_per_mtok: 3, output_usd_per_mtok: null, cached_input_usd_per_mtok: null, media_inputs: ['image', 'pdf'] },
		});
	});

	it('sends the legacy shape with a key and base URL', async () => {
		await actions.create(event({ form: { label: 'Local', provider: 'openai_compatible', model_id: 'llama', base_url: 'http://x' } }));
		expect(calls()[0].body).toMatchObject({ provider: 'openai_compatible', api_key: '', base_url: 'http://x', media_inputs: null });
		await actions.update(event({ form: { id: 'a1', label: 'Local', provider: 'openai_compatible', model_id: 'llama', api_key: 'sk' } }));
		expect(calls()[1]).toMatchObject({ path: '/api/admin/settings/llm/models/a1', method: 'PUT', body: { api_key: 'sk', base_url: null } });
		await actions.update(event({ form: { id: 'a1', label: 'S', connection_id: 'c1' } }));
		expect(calls()[2].body).toMatchObject({ connection_id: 'c1' });
	});
});

describe('admin models: the rest', () => {
	const cases: [keyof typeof actions, Record<string, string>, string, string][] = [
		['create', { label: 'S', provider: 'p' }, '/api/admin/settings/llm/models', 'POST'],
		['update', { id: 'a1', label: 'S', provider: 'p' }, '/api/admin/settings/llm/models/a1', 'PUT'],
		['delete', { id: 'a1' }, '/api/admin/settings/llm/models/a1', 'DELETE'],
		['setEnabled', { id: 'a1', enabled: 'true' }, '/api/admin/settings/llm/models/a1/enabled', 'PUT'],
		['setDefault', { id: 'a1' }, '/api/admin/settings/llm/models/a1/default', 'PUT'],
		['setFilesModel', { id: 'a1' }, '/api/admin/settings/llm/files-model', 'PUT'],
		['setCodingModel', { id: '' }, '/api/admin/settings/llm/coding-model', 'PUT'],
		['fetchModels', { provider: 'openai', api_key: 'sk' }, '/api/admin/settings/llm/models/fetch-models', 'POST'],
	];
	for (const [name, form, path, method] of cases) {
		it(`${name} calls the API and passes failures on`, async () => {
			const ok = await actions[name](event({ form }));
			expect(status(ok)).toBeUndefined();
			expect(calls()[0]).toMatchObject({ path, method });
			apiFetch.mockImplementationOnce(async () => textResponse('refused', 409));
			expect(await actions[name](event({ form }))).toMatchObject({ status: 409, data: { error: 'refused' } });
			apiFetch.mockImplementationOnce(async () => textResponse('', 500));
			expect(((await actions[name](event({ form }))) as { data: { error: string } }).data.error).toBeTruthy();
		});
	}

	it('refuses a missing id or provider', async () => {
		for (const name of ['delete', 'setEnabled', 'setDefault'] as const) expect(status(await actions[name](event({ form: {} })))).toBe(400);
		expect(status(await actions.fetchModels(event({ form: {} })))).toBe(400);
		await actions.setCodingModel(event({ form: {} }));
		expect(calls()[0].body).toEqual({ id: null });
	});

	it('lists a connection’s models and the catalog’s', async () => {
		apiFetch.mockImplementation(async () => json({ models: [{ id: 'x' }] }));
		expect(await actions.connectionModels(event({ form: { connection_id: 'c 1' } }))).toEqual({ connectionModels: [{ id: 'x' }] });
		expect(await actions.catalogModels(event({ form: { provider_id: 'anthropic' } }))).toEqual({ catalogModels: [{ id: 'x' }] });
		expect(status(await actions.connectionModels(event({ form: {} })))).toBe(400);
		expect(status(await actions.catalogModels(event({ form: {} })))).toBe(400);
	});
});
