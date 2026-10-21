import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { fetchModelOptions, manageModelActions, saveDefaultModel } from '$lib/server/modelOptions';
import type { LlmModelsResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

// The Model page: the default model for new chats (the same picker and Manage models as a
// chat's composer) and the model Koda builds with, a setting of its own.
export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [response, modelOptions] = await Promise.all([apiFetch(fetch, cookies, '/api/llm/models'), fetchModelOptions(fetch, cookies)]);
	const models: LlmModelsResponse = response.ok
		? ((await response.json()) as LlmModelsResponse)
		: { admin_models: [], coding: { kind: 'default' }, default_coding_model_id: null };
	return { models, modelOptions };
};

export const actions: Actions = {
	/** The model new chats start on (each chat can switch in its composer). */
	setDefaultModel: async ({ request, cookies, fetch }) => saveDefaultModel(fetch, cookies, await request.formData()),

	/** Which model Koda builds their projects with: 'default', 'same_as_chat' or one of Nomi's models. */
	selectCodingModel: async ({ request, cookies, fetch }) => {
		const choice = (await request.formData()).get('coding');
		if (typeof choice !== 'string' || !choice) {
			return fail(400, { error: m.err_invalid_model_selection() });
		}
		const body = choice === 'default' || choice === 'same_as_chat' ? { kind: choice } : { kind: 'admin', admin_model_id: choice };
		const response = await apiFetch(fetch, cookies, '/api/llm/coding-selection', { method: 'PUT', body: JSON.stringify(body) });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_select_model() });
		}
		return { success: true };
	},

	// Manage models: which models the picker offers, and connecting a provider in place.
	...manageModelActions,
};
