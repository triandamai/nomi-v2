import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { ProviderSettings } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm');
	if (response.status === 404) {
		return { settings: null as ProviderSettings | null };
	}
	if (!response.ok) {
		return { settings: null as ProviderSettings | null };
	}
	const settings = (await response.json()) as ProviderSettings;
	return { settings };
};

export const actions: Actions = {
	default: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const provider = data.get('provider');
		const modelId = data.get('model_id');
		const apiKey = data.get('api_key');
		const baseUrl = data.get('base_url');

		if (typeof provider !== 'string' || typeof modelId !== 'string') {
			return fail(400, { error: 'Provider and model are required.' });
		}

		const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm', {
			method: 'PUT',
			body: JSON.stringify({
				provider,
				model_id: modelId,
				api_key: typeof apiKey === 'string' && apiKey.length > 0 ? apiKey : null,
				base_url: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
			}),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to save settings.' });
		}

		const settings = (await response.json()) as ProviderSettings;
		return { settings, success: true };
	},
};
