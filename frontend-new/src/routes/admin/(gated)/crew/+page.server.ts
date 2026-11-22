import { m } from '$lib/paraglide/messages';
import type { AdminCrewMember } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/** The built-in crew (agents: view): each member, on or off for everyone. Write switches one. */
export const load: PageServerLoad = async (event) => {
	const loaded = mustLoad(await getJson<{ agents: AdminCrewMember[] }>(event, '/api/admin/crew'));
	return { crew: loaded.ok ? { ok: true as const, data: loaded.data?.agents ?? [] } : loaded };
};

export const actions: Actions = {
	toggle: async (event) => {
		const form = await event.request.formData();
		const agentType = field(form, 'agent_type');
		if (!agentType) return refuse('toggle', m.admin_agent_choose());
		return act(event, 'toggle', `/api/admin/crew/${encodeURIComponent(agentType)}`, 'PUT', { enabled: field(form, 'enabled') === 'true' });
	},
};
