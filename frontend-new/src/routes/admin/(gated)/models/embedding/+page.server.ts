import { m } from '$lib/paraglide/messages';
import type { EmbeddingSettings, FetchedModel } from '$lib/admin/api';
import { EMBEDDING_PROVIDERS } from '$lib/admin/models';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * Embedding (models: view; write changes it): the model memory and search use to read meaning, its
 * provider, key (written, never shown) and address, and finding what the provider offers.
 */
export const load: PageServerLoad = async (event) => {
	const loaded = await getJson<EmbeddingSettings>(event, '/api/admin/settings/embedding');
	// None saved: Nomi uses the one its environment sets, and one can be saved here.
	if (!loaded.ok && loaded.problem.status === 404) return { embedding: { ok: true as const, data: null } };
	return { embedding: mustLoad(loaded) };
};

const known = (provider: string) => (EMBEDDING_PROVIDERS as readonly string[]).includes(provider);

export const actions: Actions = {
	save: async (event) => {
		const form = await event.request.formData();
		const provider = field(form, 'provider');
		const modelId = field(form, 'model_id');
		if (!known(provider)) return refuse('save', m.admin_embedding_provider_invalid());
		if (!modelId) return refuse('save', m.admin_model_id_required());
		return act<EmbeddingSettings>(event, 'save', '/api/admin/settings/embedding', 'PUT', { provider, model_id: modelId, api_key: field(form, 'api_key') || null, base_url: field(form, 'base_url') || null });
	},

	findModels: async (event) => {
		const form = await event.request.formData();
		const provider = field(form, 'provider');
		if (!known(provider)) return refuse('findModels', m.admin_embedding_provider_invalid());
		return act<{ models: FetchedModel[] }>(event, 'findModels', '/api/admin/settings/embedding/fetch-models', 'POST', { provider, api_key: field(form, 'api_key') || null, base_url: field(form, 'base_url') || null });
	},
};
