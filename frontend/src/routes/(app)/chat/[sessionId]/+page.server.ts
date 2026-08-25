import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { LlmModelsResponse, MessageItem } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`);

	if (response.status === 404) {
		throw redirect(303, '/');
	}
	if (!response.ok) {
		throw error(response.status, 'Could not load this chat.');
	}

	const { messages } = (await response.json()) as { messages: MessageItem[] };

	const modelsResponse = await apiFetch(fetch, cookies, '/api/llm/models');
	const models: LlmModelsResponse = modelsResponse.ok
		? ((await modelsResponse.json()) as LlmModelsResponse)
		: { admin_models: [], selection: null };

	return { messages, models };
};

export const actions: Actions = {
	default: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const text = data.get('text');

		if (typeof text !== 'string' || !text.trim()) {
			return fail(400, { error: 'Message cannot be empty.' });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`, {
			method: 'POST',
			body: JSON.stringify({ text }),
		});

		if (response.status === 404) {
			throw redirect(303, '/');
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to send message.' });
		}

		const { user_message } = (await response.json()) as { user_message: MessageItem };

		return { user_message };
	},

	selectAdminModel: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const adminModelId = data.get('admin_model_id');

		if (typeof adminModelId !== 'string') {
			return fail(400, { modelError: 'Invalid model selection.' });
		}

		const response = await apiFetch(fetch, cookies, '/api/llm/selection', {
			method: 'PUT',
			body: JSON.stringify({ kind: 'admin', admin_model_id: adminModelId }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { modelError: message || 'Failed to select model.' });
		}

		return { modelSelected: true };
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
			return fail(400, { modelError: 'Label and provider are required.' });
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
			return fail(response.status, { modelError: message || 'Failed to validate and save this model.' });
		}

		return { modelSelected: true };
	},
};
