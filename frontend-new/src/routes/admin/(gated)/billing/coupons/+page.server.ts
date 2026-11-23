import type { AdminCoupon, AdminPlan } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

/** The coupons (plans: view) with how often each was used, searched by code with `?query=`; the plans name their limits. */
export const load: PageServerLoad = async (event) => {
	const query = (event.url.searchParams.get('query') ?? '').trim();
	const [coupons, plans] = await Promise.all([getJson<AdminCoupon[]>(event, '/api/admin/coupons'), getJson<AdminPlan[]>(event, '/api/admin/plans')]);
	const loaded = mustLoad(coupons);
	const wanted = query.toUpperCase();
	return {
		query,
		coupons: loaded.ok ? { ok: true as const, data: (loaded.data ?? []).filter((one) => !wanted || one.code.includes(wanted)) } : loaded,
		total: loaded.ok ? (loaded.data ?? []).length : 0,
		plans: plans.ok ? (plans.data ?? []).map(({ id, name }) => ({ id, name })) : [],
	};
};
