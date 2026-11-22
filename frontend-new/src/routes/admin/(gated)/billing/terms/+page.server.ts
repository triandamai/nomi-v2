import type { BillingTerm } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

/** The terms a plan can be paid for (plans: view), in their order. */
export const load: PageServerLoad = async (event) => {
	const loaded = mustLoad(await getJson<BillingTerm[]>(event, '/api/admin/billing/terms'));
	return { terms: loaded.ok ? { ok: true as const, data: (loaded.data ?? []).toSorted((a, b) => a.sort_order - b.sort_order || a.months - b.months) } : loaded };
};
