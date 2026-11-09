import { apiFetch } from '$lib/server/api';
import type { DashboardStats } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/dashboard');
	const stats: DashboardStats = response.ok
		? ((await response.json()) as DashboardStats)
		: { total_users: 0, tokens_today: 0, tokens_all_time: 0, running_agents: 0, sandbox: { running: 0, starting: 0, error: 0, minutes_this_month: 0 } };
	return { stats };
};
