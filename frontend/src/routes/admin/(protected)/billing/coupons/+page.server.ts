import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { AdminCoupon, Plan } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';
import { failure } from '../failure';
import { readCoupon, type CouponFormProblem } from './couponForm';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [couponsResponse, plansResponse] = await Promise.all([apiFetch(fetch, cookies, '/api/admin/coupons'), apiFetch(fetch, cookies, '/api/admin/plans')]);
	const coupons: AdminCoupon[] = couponsResponse.ok ? ((await couponsResponse.json()) as AdminCoupon[]) : [];
	// The plans a coupon can be limited to; the picker is empty if the list can't load.
	const plans: Pick<Plan, 'id' | 'name'>[] = plansResponse.ok ? ((await plansResponse.json()) as Plan[]).map(({ id, name }) => ({ id, name })) : [];
	return { coupons, plans };
};

const PROBLEMS: Record<CouponFormProblem, () => string> = {
	code: () => m.billing_admin_coupon_bad_code(),
	value: () => m.billing_admin_coupon_bad_value(),
	limits: () => m.billing_admin_coupon_bad_limits(),
	dates: () => m.billing_admin_coupon_bad_dates(),
};

export const actions: Actions = {
	save: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const coupon = readCoupon(data);
		if (!coupon.ok) return fail(400, { error: PROBLEMS[coupon.problem]() });
		const id = data.get('id');
		const editing = typeof id === 'string' && id.length > 0;
		const response = await apiFetch(fetch, cookies, editing ? `/api/admin/coupons/${id}` : '/api/admin/coupons', {
			method: editing ? 'PUT' : 'POST',
			body: JSON.stringify(coupon.payload),
		});
		return response.ok ? { saved: true } : failure(response, m.billing_admin_save_failed());
	},

	// A coupon somebody used can't be deleted; the backend says so, and the page shows its words.
	delete: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		const response = await apiFetch(fetch, cookies, `/api/admin/coupons/${id}`, { method: 'DELETE' });
		return response.ok ? { success: true } : failure(response, m.billing_admin_delete_failed());
	},
};
