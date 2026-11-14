import { error, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { AdminModel, CatalogProvider, Connection, FetchedModel } from '$lib/admin/api';
import { modelFromForm } from '$lib/admin/models';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * A chat model to add (`/admin/models/new`) or change (models: write; view shows it): on a
 * connected provider or with its own key, its id (found from what the provider serves), prices and
 * what it opens. There's no single-model endpoint: an existing one comes from the list.
 */
export const load: PageServerLoad = async (event) => {
	const isNew = event.params.id === 'new';
	const [list, connections, catalog] = await Promise.all([
		isNew ? null : getJson<AdminModel[]>(event, '/api/admin/settings/llm/models'),
		getJson<{ connections: Connection[] }>(event, '/api/admin/settings/llm/connections'),
		getJson<{ providers: CatalogProvider[] }>(event, '/api/llm/providers'),
	]);
	const base = { connections: connections.ok ? connections.data.connections : [], providers: catalog.ok ? catalog.data.providers : [] };
	if (!list) return { title: m.admin_model_new(), model: null, ...base };
	const loaded = mustLoad(list);
	if (!loaded.ok) return { title: m.admin_page_models(), model: loaded, ...base };
	const model = loaded.data.find((one) => one.id === event.params.id);
	if (!model) error(404, { message: m.admin_model_missing() });
	return { title: model.label, model: { ok: true as const, data: model }, ...base };
};

export const actions: Actions = {
	save: async (event) => {
		const editing = event.params.id !== 'new';
		const read = modelFromForm(await event.request.formData(), { editing });
		if (!read.ok) return refuse('save', read.message);
		if (editing) return act(event, 'save', `/api/admin/settings/llm/models/${encodeURIComponent(event.params.id)}`, 'PUT', read.body);
		const created = await act(event, 'save', '/api/admin/settings/llm/models', 'POST', read.body);
		if ('ok' in created && created.ok) redirect(303, '/admin/models');
		return created;
	},

	/** What the provider serves now: a connected one by its own key, or the key and address typed here. */
	findModels: async (event) => {
		const form = await event.request.formData();
		if (field(form, 'source') !== 'own') {
			const connection = field(form, 'connection_id');
			if (!connection) return refuse('findModels', m.admin_model_connection_required());
			return act<{ models: FetchedModel[] }>(event, 'findModels', `/api/admin/settings/llm/connections/${encodeURIComponent(connection)}/models`, 'POST');
		}
		const provider = field(form, 'provider');
		if (!provider) return refuse('findModels', m.admin_model_provider_required());
		return act<{ models: FetchedModel[] }>(event, 'findModels', '/api/admin/settings/llm/models/fetch-models', 'POST', {
			provider,
			api_key: field(form, 'api_key') || null,
			base_url: field(form, 'base_url') || null,
			...(event.params.id !== 'new' && { existing_model_id: event.params.id }),
		});
	},

	delete: async (event) => {
		const deleted = await act(event, 'delete', `/api/admin/settings/llm/models/${encodeURIComponent(event.params.id)}`, 'DELETE');
		if ('ok' in deleted && deleted.ok) redirect(303, '/admin/models');
		return deleted;
	},
};
