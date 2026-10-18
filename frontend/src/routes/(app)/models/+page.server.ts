import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import {
	connectProvider,
	listConnectionModels,
	loadCatalog,
	loadConnections,
	PERSONAL_CONNECTIONS,
	selectConnectionModel,
} from '$lib/server/providers';
import type { LlmModelsResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [response, connections, providers] = await Promise.all([
		apiFetch(fetch, cookies, '/api/llm/models'),
		loadConnections(fetch, cookies, PERSONAL_CONNECTIONS),
		loadCatalog(fetch, cookies),
	]);
	const models: LlmModelsResponse = response.ok
		? ((await response.json()) as LlmModelsResponse)
		: { admin_models: [], selection: null, coding: { kind: 'default' }, default_coding_model_id: null };
	return { models, connections: connections ?? [], providers: providers ?? [] };
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

	// Your own providers: connect one in place, list its models, use one.
	connect: async ({ request, cookies, fetch }) => connectProvider(fetch, cookies, PERSONAL_CONNECTIONS, await request.formData()),
	connectionModels: async ({ request, cookies, fetch }) => listConnectionModels(fetch, cookies, PERSONAL_CONNECTIONS, await request.formData()),
	selectConnectionModel: async ({ request, cookies, fetch }) => selectConnectionModel(fetch, cookies, await request.formData()),
};
