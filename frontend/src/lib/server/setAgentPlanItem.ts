import { error, type Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';

/** Ticks or unticks one step of a plan version's checklist (counted as $lib/planChecklist does). */
export async function setAgentPlanItem(
	fetch: typeof globalThis.fetch,
	cookies: Cookies,
	sessionId: string,
	agentSessionId: string,
	planId: string,
	index: number,
	done: boolean,
): Promise<void> {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/agent-plans/${agentSessionId}/${planId}/items/${index}`, {
		method: 'PUT',
		body: JSON.stringify({ done }),
	});
	if (!response.ok) throw error(response.status, m.err_save_plan_item());
}
