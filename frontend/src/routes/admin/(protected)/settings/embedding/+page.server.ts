import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export type EmbeddingSettings = {
	provider: string;
	model_id: string;
	base_url: string | null;
	api_key_masked: string;
};

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/settings/embedding');
	if (!response.ok) {
		return { settings: null as EmbeddingSettings | null };
	}
	const settings = (await response.json()) as EmbeddingSettings;
	return { settings };
};

export const actions: Actions = {
	update: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const provider = data.get('provider');
		const modelId = data.get('model_id');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');
		if (typeof provider !== 'string' || !provider) {
			return fail(400, { error: 'Provider is required.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/admin/settings/embedding', {
			method: 'PUT',
			body: JSON.stringify({
				provider,
				model_id: typeof modelId === 'string' ? modelId : '',
				api_key: typeof apiKey === 'string' && apiKey.length > 0 ? apiKey : null,
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
			}),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to update embedding settings.' });
		}
		return { success: true };
	},

	fetchModels: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const provider = data.get('provider');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');
		if (typeof provider !== 'string' || !provider) {
			return fail(400, { error: 'Provider is required.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/admin/settings/embedding/fetch-models', {
			method: 'POST',
			body: JSON.stringify({
				provider,
				api_key: typeof apiKey === 'string' && apiKey.length > 0 ? apiKey : null,
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
			}),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Could not fetch models — enter the model ID manually.' });
		}
		const result = (await response.json()) as { models: { id: string; label: string | null }[] };
		return { models: result.models };
	},
};
