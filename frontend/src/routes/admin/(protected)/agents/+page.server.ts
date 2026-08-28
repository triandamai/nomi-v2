import { apiFetch } from '$lib/server/api';
import type { AgentsResponse } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/agents');
	const agents: AgentsResponse = response.ok ? ((await response.json()) as AgentsResponse) : { users: [] };
	return { agents };
};
