import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { Actions, PageServerLoad } from './$types';

/** A built-in crew member as Admin → Agents lists it. */
export interface AdminCrewMember {
	agent_type: string;
	name: string;
	role: string;
	/** Switched on for everyone. */
	enabled: boolean;
	/** It can't be switched off (Nomi). */
	locked: boolean;
}

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/crew');
	if (!response.ok) return { agents: [] as AdminCrewMember[] };
	const body = (await response.json()) as { agents?: AdminCrewMember[] };
	return { agents: Array.isArray(body.agents) ? body.agents : [] };
};

export const actions: Actions = {
	toggle: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const agentType = data.get('agent_type');
		if (typeof agentType !== 'string' || !agentType) {
			return fail(400, { error: m.err_invalid_agent() });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/crew/${encodeURIComponent(agentType)}`, {
			method: 'PUT',
			body: JSON.stringify({ enabled: data.get('enabled') === 'true' }),
		});
		if (!response.ok) {
			return fail(response.status, { error: m.err_toggle_agent() });
		}
		return { success: true };
	},
};
