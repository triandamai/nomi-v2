import { m } from '$lib/paraglide/messages';
import type { CatalogProvider, Connection } from '$lib/admin/api';
import { headersFromText } from '$lib/admin/models';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * Providers (models: view): the AI providers connected for everyone, with the models that use each,
 * and (write) connecting another from the catalog, changing one, or (manage) disconnecting one no
 * model uses. Keys are written, never shown.
 */
export const load: PageServerLoad = async (event) => {
	const [connections, catalog] = await Promise.all([
		getJson<{ connections: Connection[] }>(event, '/api/admin/settings/llm/connections'),
		getJson<{ providers: CatalogProvider[] }>(event, '/api/llm/providers'),
	]);
	const listed = mustLoad(connections);
	return { connections: listed.ok ? { ok: true as const, data: listed.data.connections } : listed, providers: catalog.ok ? catalog.data.providers : [] };
};

export const actions: Actions = {
	connect: async (event) => {
		const form = await event.request.formData();
		const providerId = field(form, 'provider_id');
		if (!providerId) return refuse('connect', m.admin_provider_required());
		const headers = headersFromText(field(form, 'headers'));
		if (!headers) return refuse('connect', m.admin_provider_headers_invalid());
		return act(event, 'connect', '/api/admin/settings/llm/connections', 'POST', {
			provider_id: providerId,
			...(field(form, 'label') && { label: field(form, 'label') }),
			...(field(form, 'api_key') && { api_key: field(form, 'api_key') }),
			...(field(form, 'base_url') && { base_url: field(form, 'base_url') }),
			...(headers.length > 0 && { headers }),
		});
	},

	update: async (event) => {
		const form = await event.request.formData();
		const id = field(form, 'id');
		if (!id) return refuse('update', m.admin_provider_required());
		// A new key or address is checked live by the backend: send only what changed.
		const baseUrl = field(form, 'base_url');
		return act(event, 'update', `/api/admin/settings/llm/connections/${encodeURIComponent(id)}`, 'PUT', {
			...(field(form, 'label') && { label: field(form, 'label') }),
			...(field(form, 'api_key') && { api_key: field(form, 'api_key') }),
			...(baseUrl && baseUrl !== field(form, 'was_base_url') && { base_url: baseUrl }),
		});
	},

	disconnect: async (event) => {
		const id = field(await event.request.formData(), 'id');
		if (!id) return refuse('disconnect', m.admin_provider_required());
		return act(event, 'disconnect', `/api/admin/settings/llm/connections/${encodeURIComponent(id)}`, 'DELETE');
	},
};
