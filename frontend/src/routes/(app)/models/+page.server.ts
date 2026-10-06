import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { LlmModelsResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/llm/models');
	const models: LlmModelsResponse = response.ok
		? ((await response.json()) as LlmModelsResponse)
		: { admin_models: [], selection: null };
	return { models };
};

export const actions: Actions = {
	selectAdminModel: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const adminModelId = data.get('admin_model_id');

		if (typeof adminModelId !== 'string') {
			return fail(400, { error: m.err_invalid_model_selection() });
		}

		const response = await apiFetch(fetch, cookies, '/api/llm/selection', {
			method: 'PUT',
			body: JSON.stringify({ kind: 'admin', admin_model_id: adminModelId }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_select_model() });
		}
		return { success: true };
	},

	selectCustomModel: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const label = data.get('label');
		const provider = data.get('provider');
		const modelId = data.get('model_id');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');

		if (
			typeof label !== 'string' ||
			typeof provider !== 'string' ||
			typeof modelId !== 'string' ||
			typeof apiKey !== 'string' ||
			!label.trim() ||
			!provider.trim()
		) {
			return fail(400, { error: m.err_label_provider_required() });
		}

		const response = await apiFetch(fetch, cookies, '/api/llm/selection', {
			method: 'PUT',
			body: JSON.stringify({
				kind: 'custom',
				label,
				provider,
				model_id: modelId,
				api_key: apiKey,
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
			}),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_validate_model() });
		}
		return { success: true };
	},

	fetchModels: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const provider = data.get('provider');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');

		if (typeof provider !== 'string' || !provider) {
			return fail(400, { error: m.err_provider_required() });
		}

		const response = await apiFetch(fetch, cookies, '/api/llm/fetch-models', {
			method: 'POST',
			body: JSON.stringify({
				provider,
				api_key: typeof apiKey === 'string' ? apiKey : '',
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
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
