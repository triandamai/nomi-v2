import { apiFetch } from '$lib/server/api';
import type { AgentEventItem, AgentsResponse } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const agentsResponse = await apiFetch(fetch, cookies, '/api/admin/agents');
	const agents: AgentsResponse = agentsResponse.ok ? ((await agentsResponse.json()) as AgentsResponse) : { users: [] };

	const eventsResponse = await apiFetch(fetch, cookies, '/api/admin/agent-events');
	const events: AgentEventItem[] = eventsResponse.ok ? ((await eventsResponse.json()) as AgentEventItem[]) : [];

	return { agents, events };
};
