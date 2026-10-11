import { apiFetch } from '$lib/server/api';
import type { AdminUsage } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const month = url.searchParams.get('month');
	const query = month && /^\d{4}-\d{2}$/.test(month) ? `?month=${month}` : '';
	const response = await apiFetch(fetch, cookies, `/api/admin/usage${query}`);
	return { usage: response.ok ? ((await response.json()) as AdminUsage) : null };
};
