import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { AdminCoupon, AdminPlan } from '$lib/admin/api';
import { couponFromForm } from '$lib/admin/billing';
import { act, getJson, mustLoad, refuse, send } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/** A coupon to add (`/admin/billing/coupons/new`) or change (plans: write), with the plans it can be limited to. */
export const load: PageServerLoad = async (event) => {
	const plansLoaded = await getJson<AdminPlan[]>(event, '/api/admin/plans');
	const plans = plansLoaded.ok ? (plansLoaded.data ?? []).map(({ id, name }) => ({ id, name })) : [];
	if (event.params.id === 'new') return { title: m.admin_coupon_new(), coupon: null, plans };
	const list = mustLoad(await getJson<AdminCoupon[]>(event, '/api/admin/coupons'));
	if (!list.ok) return { title: m.admin_page_coupons(), coupon: list, plans };
	const coupon = list.data.find((one) => one.id === event.params.id);
	if (!coupon) error(404, { message: m.admin_coupon_missing() });
	return { title: coupon.code, coupon: { ok: true as const, data: coupon }, plans };
};

export const actions: Actions = {
	save: async (event) => {
		const read = couponFromForm(await event.request.formData());
		if (!read.ok) return refuse('save', read.message);
		if (event.params.id !== 'new') return act(event, 'save', `/api/admin/coupons/${encodeURIComponent(event.params.id)}`, 'PUT', read.input);
		const created = await send(event, '/api/admin/coupons', 'POST', read.input);
		if (!created.ok) return fail(created.problem.status >= 400 ? created.problem.status : 500, { action: 'save', ok: false as const, problem: created.problem });
		redirect(303, '/admin/billing/coupons');
	},

	/** Only a coupon nobody used can go; the backend says so otherwise, and the page shows its words. */
	delete: async (event) => {
		const deleted = await act(event, 'delete', `/api/admin/coupons/${encodeURIComponent(event.params.id)}`, 'DELETE');
		if ('ok' in deleted && deleted.ok) redirect(303, '/admin/billing/coupons');
		return deleted;
	},
};
