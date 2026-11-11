import { m } from '$lib/paraglide/messages';
import type { DynamicAgent } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/** Custom agents (agents: view; write changes): the crew members made here, each on or off. */
export const load: PageServerLoad = async (event) => ({ agents: mustLoad(await getJson<DynamicAgent[]>(event, '/api/admin/dynamic-agents')) });

export const actions: Actions = {
	toggle: async (event) => {
		const id = field(await event.request.formData(), 'id');
		if (!id) return refuse('toggle', m.admin_agent_choose());
		return act<DynamicAgent>(event, 'toggle', `/api/admin/dynamic-agents/${encodeURIComponent(id)}/toggle-active`, 'POST');
	},
};
