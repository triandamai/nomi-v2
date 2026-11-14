import type { AgentEvent, AgentsResponse } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/**
 * Live agents (dashboard: view): who is working now and the latest events; the page keeps both
 * current from the admin stream. `?agent=&session=` opens one agent's steps: the events of its
 * chat (the backend filters by chat, not by agent), kept to its own.
 */
export const load: PageServerLoad = async (event) => {
	const [agents, events] = await Promise.all([getJson<AgentsResponse>(event, '/api/admin/agents'), getJson<AgentEvent[]>(event, '/api/admin/agent-events?limit=50')]);
	mustLoad(agents);

	const agent = event.url.searchParams.get('agent');
	const session = event.url.searchParams.get('session');
	let detail = null;
	if (agent && session && UUID.test(session)) {
		const loaded = await getJson<AgentEvent[]>(event, `/api/admin/agent-events?session_id=${session}&limit=200`);
		detail = { agent, session, events: loaded.ok ? { ok: true as const, data: loaded.data.filter((each) => each.agent_session_id === agent) } : loaded };
	}
	return { agents, events, detail };
};
