import { apiFetch } from '$lib/server/api';
import type { PlansForUser, UsageMonth } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const month = url.searchParams.get('month');
	const query = month && /^\d{4}-\d{2}$/.test(month) ? `?month=${month}` : '';
	const [usageResponse, plansResponse] = await Promise.all([apiFetch(fetch, cookies, `/api/usage${query}`), apiFetch(fetch, cookies, '/api/plans')]);
	const usage: UsageMonth | null = usageResponse.ok ? ((await usageResponse.json()) as UsageMonth) : null;
	const plans: PlansForUser | null = plansResponse.ok ? ((await plansResponse.json()) as PlansForUser) : null;
	return { usage, plans };
};
