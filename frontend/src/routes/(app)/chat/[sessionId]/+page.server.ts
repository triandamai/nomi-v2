import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import type { LlmModelsResponse, MessageItem, PersonalityHistoryResponse, RenderedMessage } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`);

	if (response.status === 404) {
		throw redirect(303, '/');
	}
	if (!response.ok) {
		throw error(response.status, 'Could not load this chat.');
	}

	const { messages: rawMessages } = (await response.json()) as { messages: MessageItem[] };
	const messages: RenderedMessage[] = await Promise.all(
		rawMessages.map(async (message) => ({ ...message, content_html: await renderMarkdown(message.content) })),
	);

	const modelsResponse = await apiFetch(fetch, cookies, '/api/llm/models');
	const models: LlmModelsResponse = modelsResponse.ok
		? ((await modelsResponse.json()) as LlmModelsResponse)
		: { admin_models: [], selection: null };

	const personalityResponse = await apiFetch(fetch, cookies, '/api/personality/history');
	const personality: PersonalityHistoryResponse = personalityResponse.ok
		? ((await personalityResponse.json()) as PersonalityHistoryResponse)
		: { versions: [] };

	const agentActivityResponse = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/agent-activity`);
	const agentActivity = agentActivityResponse.ok ? await agentActivityResponse.json() : [];

	return { messages, models, personality, agentActivity };
};

export const actions: Actions = {
	// Named (not `default`) because this actions object also has selectAdminModel and
	// selectCustomModel — SvelteKit forbids mixing a `default` action with named actions in the
	// same file (throws "When using named actions, the default action cannot be used" at request
	// time), so the message-send form below must target this action explicitly.
	sendMessage: async ({ request, params, cookies, fetch }) => {
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

	feedback: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const messageId = data.get('messageId');
		const rating = data.get('rating');

		if (typeof messageId !== 'string' || !messageId) {
			return fail(400, { error: 'Invalid message.' });
		}

		const path = `/api/sessions/${params.sessionId}/messages/${messageId}/feedback`;
		const response =
			rating === 'up' || rating === 'down'
				? await apiFetch(fetch, cookies, path, { method: 'PUT', body: JSON.stringify({ rating }) })
				: await apiFetch(fetch, cookies, path, { method: 'DELETE' });

		if (!response.ok) {
			return fail(response.status, { error: 'Failed to save feedback.' });
		}
		return { success: true };
	},

	restorePersonality: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const version = data.get('version');

		if (typeof version !== 'string' || !version.trim()) {
			return fail(400, { personalityError: 'Invalid version.' });
		}

		const response = await apiFetch(fetch, cookies, '/api/personality/rollback', {
			method: 'POST',
			body: JSON.stringify({ version: Number(version) }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { personalityError: message || 'Failed to roll back personality.' });
		}

		return { personalityRestored: true };
	},
};
