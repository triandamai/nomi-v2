import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { loadThinkingLevel, saveThinkingLevel } from '$lib/server/thinking';
import { renderMarkdown } from '$lib/server/markdown';
import type { AgentStatus, LlmModelsResponse, MessageItem, PersonalityHistoryResponse, RenderedMessage, SessionSummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`);

	if (response.status === 404) {
		throw redirect(303, '/');
	}
	if (!response.ok) {
		throw error(response.status, m.err_load_chat());
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

	const agentStatusResponse = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/agent-status`);
	const agentStatus: AgentStatus | null = agentStatusResponse.ok ? await agentStatusResponse.json() : null;

	// There's no single-session endpoint; the title comes from the session list (best-effort).
	const sessionsResponse = await apiFetch(fetch, cookies, '/api/sessions');
	const sessions: SessionSummary[] = sessionsResponse.ok
		? ((await sessionsResponse.json()) as { sessions: SessionSummary[] }).sessions
		: [];
	const title = sessions.find((s) => s.id === params.sessionId)?.title ?? m.nav_new_chat();

	const thinkingLevel = await loadThinkingLevel(fetch, cookies, params.sessionId);
	return { messages, models, personality, agentActivity, agentStatus, title, thinkingLevel };
};

export const actions: Actions = {
	setThinking: async ({ request, params, cookies, fetch }) => saveThinkingLevel(fetch, cookies, params.sessionId, await request.formData()),

	// Named (not `default`) because this actions object also has selectAdminModel and
	// selectCustomModel — SvelteKit forbids mixing a `default` action with named actions in the
	// same file (throws "When using named actions, the default action cannot be used" at request
	// time), so the message-send form below must target this action explicitly.
	sendMessage: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const text = data.get('text');

		if (typeof text !== 'string' || !text.trim()) {
			return fail(400, { error: m.err_empty_message() });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`, {
			method: 'POST',
			body: JSON.stringify({ text }),
		});

		if (response.status === 404) {
			throw redirect(303, '/');
		}
		if (!response.ok) {
			return fail(response.status, { error: m.err_send_message() });
		}

		const { user_message, supervisor_reply } = (await response.json()) as {
			user_message: MessageItem;
			supervisor_reply?: MessageItem;
		};

		// A stop command is answered on the spot by the supervisor, so no turn is coming.
		return { user_message, stopped: supervisor_reply !== undefined };
	},

	selectAdminModel: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const adminModelId = data.get('admin_model_id');

		if (typeof adminModelId !== 'string') {
			return fail(400, { modelError: m.err_invalid_model_selection() });
		}

		const response = await apiFetch(fetch, cookies, '/api/llm/selection', {
			method: 'PUT',
			body: JSON.stringify({ kind: 'admin', admin_model_id: adminModelId }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { modelError: message || m.err_select_model() });
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
			return fail(400, { modelError: m.err_label_provider_required() });
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
			return fail(response.status, { modelError: message || m.err_validate_model() });
		}

		return { modelSelected: true };
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

	feedback: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const messageId = data.get('messageId');
		const rating = data.get('rating');
		const reason = data.get('reason');

		if (typeof messageId !== 'string' || !messageId) {
			return fail(400, { error: m.err_invalid_message() });
		}

		const path = `/api/sessions/${params.sessionId}/messages/${messageId}/feedback`;
		const response =
			rating === 'up' || rating === 'down'
				? await apiFetch(fetch, cookies, path, { method: 'PUT', body: JSON.stringify({ rating, reason: typeof reason === 'string' && reason ? reason : null }) })
				: await apiFetch(fetch, cookies, path, { method: 'DELETE' });

		if (!response.ok) {
			return fail(response.status, { error: m.err_save_feedback() });
		}
		return { success: true };
	},

	resolveApproval: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const messageId = data.get('messageId');
		const decision = data.get('decision');
		const remember = data.get('remember') === 'true';

		if (typeof messageId !== 'string' || (decision !== 'approve' && decision !== 'deny')) {
			return fail(400, { error: m.err_invalid_approval() });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages/${messageId}/approval`, {
			method: 'PUT',
			body: JSON.stringify({ decision, remember }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_record_decision() });
		}

		return { success: true };
	},

	restorePersonality: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const version = data.get('version');

		if (typeof version !== 'string' || !version.trim()) {
			return fail(400, { personalityError: m.err_invalid_version() });
		}

		const response = await apiFetch(fetch, cookies, '/api/personality/rollback', {
			method: 'POST',
			body: JSON.stringify({ version: Number(version) }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { personalityError: message || m.err_rollback_personality() });
		}

		return { personalityRestored: true };
	},
};
