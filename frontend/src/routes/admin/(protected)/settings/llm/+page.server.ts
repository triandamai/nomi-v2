import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { AdminLlmModel } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm/models');
	if (!response.ok) {
		return { models: [] as AdminLlmModel[] };
	}
	const models = (await response.json()) as AdminLlmModel[];
	return { models };
};

function readForm(data: FormData) {
	const label = data.get('label');
	const provider = data.get('provider');
	const modelId = data.get('model_id');
	const apiKey = data.get('api_key');
	const baseUrl = data.get('base_url');
	return {
		label: typeof label === 'string' ? label : '',
		provider: typeof provider === 'string' ? provider : '',
		modelId: typeof modelId === 'string' ? modelId : '',
		apiKey: typeof apiKey === 'string' && apiKey.length > 0 ? apiKey : null,
		baseUrl: typeof baseUrl === 'string' && baseUrl.length > 0 ? baseUrl : null,
	};
}

export const actions: Actions = {
	create: async ({ request, cookies, fetch }) => {
		const { label, provider, modelId, apiKey, baseUrl } = readForm(await request.formData());
		if (!label || !provider) {
			return fail(400, { error: 'Label and provider are required.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/admin/settings/llm/models', {
			method: 'POST',
			body: JSON.stringify({ label, provider, model_id: modelId, api_key: apiKey ?? '', base_url: baseUrl }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to create model.' });
		}
		return { success: true };
	},

	update: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		const { label, provider, modelId, apiKey, baseUrl } = readForm(data);
		if (typeof id !== 'string' || !label || !provider) {
			return fail(400, { error: 'Label and provider are required.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}`, {
			method: 'PUT',
			body: JSON.stringify({ label, provider, model_id: modelId, api_key: apiKey, base_url: baseUrl }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to update model.' });
		}
		return { success: true };
	},

	delete: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') {
			return fail(400, { error: 'Invalid model.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}`, { method: 'DELETE' });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to delete model.' });
		}
		return { success: true };
	},

	setDefault: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') {
			return fail(400, { error: 'Invalid model.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/settings/llm/models/${id}/default`, { method: 'PUT' });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to set default.' });
		}
		return { success: true };
	},
};
