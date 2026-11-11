import type { AdminUsage } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

/** Usage by month (dashboard: view): `GET /api/admin/usage?month=YYYY-MM`, this month without one. */
export const load: PageServerLoad = async (event) => {
	const month = event.url.searchParams.get('month');
	const query = month && /^\d{4}-\d{2}$/.test(month) ? `?month=${month}` : '';
	return { usage: mustLoad(await getJson<AdminUsage>(event, `/api/admin/usage${query}`)) };
};
