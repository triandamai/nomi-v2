import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { ADMIN_CONNECTIONS, listCatalogModels, listConnectionModels, loadCatalog, loadConnections } from '$lib/server/providers';
import type { AdminLlmModel } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	// New models run on a shared provider connection (Providers page); the catalog names them.
	const [response, connections, providers] = await Promise.all([
		apiFetch(fetch, cookies, '/api/admin/settings/llm/models'),
		loadConnections(fetch, cookies, ADMIN_CONNECTIONS),
		loadCatalog(fetch, cookies),
	]);
	const models = response.ok ? ((await response.json()) as AdminLlmModel[]) : [];
	return { models, connections: connections ?? [], providers: providers ?? [] };
};

function readForm(data: FormData) {
	const label = data.get('label');
	const provider = data.get('provider');
	const modelId = data.get('model_id');
	const apiKey = data.get('api_key');
	const baseUrl = data.get('base_url');
	// A price left blank stays unset; anything else must be a number of dollars, zero or more.
	const price = (name: string) => {
		const raw = data.get(name);
		if (typeof raw !== 'string' || raw.trim() === '') return null;
		const value = Number(raw);
		return Number.isFinite(value) && value >= 0 ? value : NaN;
	};
	// "auto" leaves it to the backend to work out what the model opens from its id.
	const mediaInputs = data.get('media_mode') === 'listed' ? data.getAll('media').filter((v): v is string => typeof v === 'string') : null;
	const connectionId = data.get('connection_id');
	return {
		connectionId: typeof connectionId === 'string' && connectionId.length > 0 ? connectionId : null,
		mediaInputs,
		inputPrice: price('input_usd_per_mtok'),
		outputPrice: price('output_usd_per_mtok'),
		cachedPrice: price('cached_input_usd_per_mtok'),
		label: typeof label === 'string' ? label : '',
		provider: typeof provider === 'string' ? provider : '',
		modelId: typeof modelId === 'string' ? modelId : '',
		apiKey: typeof apiKey === 'string' && apiKey.length > 0 ? apiKey : null,
		baseUrl: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
	};
}

export const actions: Actions = {
	create: async ({ request, cookies, fetch }) => {
		const { label, provider, connectionId, modelId, apiKey, baseUrl, inputPrice, outputPrice, cachedPrice, mediaInputs } = readForm(await request.formData());
		if (!label || (!connectionId && !provider)) {
			return fail(400, { error: m.err_label_provider_required() });
		}
		if (Number.isNaN(inputPrice) || Number.isNaN(outputPrice) || Number.isNaN(cachedPrice)) {
			return fail(400, { error: m.err_price() });
		}
		const prices = { input_usd_per_mtok: inputPrice, output_usd_per_mtok: outputPrice, cached_input_usd_per_mtok: cachedPrice, media_inputs: mediaInputs };
		// A model on a provider connection carries no key of its own; the legacy shape still works.
		const body = connectionId
			? { label, connection_id: connectionId, model_id: modelId, ...prices }
			: { label, provider, model_id: modelId, api_key: apiKey ?? '', base_url: baseUrl, ...prices };
		const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm/models', {
			method: 'POST',
			body: JSON.stringify(body),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_create_model() });
		}
		return { success: true };
	},

	update: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		const { label, provider, connectionId, modelId, apiKey, baseUrl, inputPrice, outputPrice, cachedPrice, mediaInputs } = readForm(data);
		if (typeof id !== 'string' || !label || (!connectionId && !provider)) {
			return fail(400, { error: m.err_label_provider_required() });
		}
		if (Number.isNaN(inputPrice) || Number.isNaN(outputPrice) || Number.isNaN(cachedPrice)) {
			return fail(400, { error: m.err_price() });
		}
		const prices = { input_usd_per_mtok: inputPrice, output_usd_per_mtok: outputPrice, cached_input_usd_per_mtok: cachedPrice, media_inputs: mediaInputs };
		const body = connectionId
			? { label, connection_id: connectionId, model_id: modelId, ...prices }
			: { label, provider, model_id: modelId, api_key: apiKey, base_url: baseUrl, ...prices };
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}`, {
			method: 'PUT',
			body: JSON.stringify(body),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_update_model() });
		}
		return { success: true };
	},

	delete: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') {
			return fail(400, { error: m.err_invalid_model() });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}`, { method: 'DELETE' });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_delete_model() });
		}
		return { success: true };
	},

	// Switches a model on or off for everyone (the default can't be switched off).
	setEnabled: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') {
			return fail(400, { error: m.err_invalid_model() });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}/enabled`, {
			method: 'PUT',
			body: JSON.stringify({ enabled: data.get('enabled') === 'true' }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_set_default() });
		}
		return { success: true };
	},

	setDefault: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') {
			return fail(400, { error: m.err_invalid_model() });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}/default`, { method: 'PUT' });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_set_default() });
		}
		return { success: true };
	},

	/** Picks the model that reads files people's own models can't open; no id stops using one. */
	setFilesModel: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm/files-model', {
			method: 'PUT',
			body: JSON.stringify({ id: typeof id === 'string' && id.length > 0 ? id : null }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_files_model() });
		}
		return { success: true };
	},

	/** Picks the model Koda builds projects with; no id leaves Koda on each person's chat model. */
	setCodingModel: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm/coding-model', {
			method: 'PUT',
			body: JSON.stringify({ id: typeof id === 'string' && id.length > 0 ? id : null }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_coding_model() });
		}
		return { success: true };
	},

	/** The models a shared connection offers, asked of the provider with its stored key. */
	connectionModels: async ({ request, cookies, fetch }) => listConnectionModels(fetch, cookies, ADMIN_CONNECTIONS, await request.formData()),

	/** The catalog's prices and inputs for a provider's models, to prefill a new one. */
	catalogModels: async ({ request, cookies, fetch }) => listCatalogModels(fetch, cookies, await request.formData()),

	fetchModels: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const provider = data.get('provider');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');
		const existingModelId = data.get('existing_model_id');
		if (typeof provider !== 'string' || !provider) {
			return fail(400, { error: m.err_provider_required() });
		}
		const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm/models/fetch-models', {
			method: 'POST',
			body: JSON.stringify({
				provider,
				api_key: typeof apiKey === 'string' && apiKey.length > 0 ? apiKey : null,
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
				existing_model_id: typeof existingModelId === 'string' && existingModelId.length > 0 ? existingModelId : null,
			}),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_fetch_models() });
		}
		const result = (await response.json()) as { models: { id: string; label: string | null }[] };
		return { models: result.models };
	},
};
