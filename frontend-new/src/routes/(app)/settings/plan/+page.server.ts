import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { joinWaitlist, loadPayment, loadPlan, startCheckout } from '$lib/server/plan';
import type { Actions, PageServerLoad } from './$types';

/**
 * Settings › Plan & usage (App / Settings / Plan & usage C0og49); `?month=YYYY-MM` for another
 * month, `?payment=<id>` coming back from the payment gateway (it returns to /billing, which comes
 * here with its query).
 */
export const load: PageServerLoad = async (event) => {
	const paymentId = event.url.searchParams.get('payment');
	const [data, payment] = await Promise.all([loadPlan(event, event.url.searchParams.get('month')), paymentId ? loadPayment(event, paymentId) : null]);
	if (!data) error(503, { message: m.usage_load_failed() });
	// The top bar's Export usage CSV, for the month shown.
	return { title: m.settings_plan(), csv: `/settings/plan/usage.csv?month=${data.usage.month}`, payment, ...data };
};

const CHECKOUT_REFUSED: Record<string, () => string> = {
	payments_not_configured: m.plan_checkout_unconfigured,
	already_on_plan: m.plan_checkout_on_plan,
	not_for_sale: m.plan_checkout_not_for_sale,
};

const planOf = async (request: Request) => String((await request.formData()).get('plan') ?? '').trim();

export const actions: Actions = {
	/** Tell me when this plan opens (one without a price). */
	waitlist: async (event) => {
		const plan = await planOf(event.request);
		if (!plan) return fail(400, { message: m.plan_pick() });
		return (await joinWaitlist(event, plan)) ? { waitlisted: plan } : fail(502, { message: m.plan_waitlist_failed() });
	},
	/** Off to the gateway's page to pay a month of the plan; a native form post, since it's another site. */
	checkout: async (event) => {
		const plan = await planOf(event.request);
		if (!plan) return fail(400, { message: m.plan_pick() });
		const started = await startCheckout(event, plan);
		if ('url' in started) redirect(303, started.url);
		return fail(started.status, { message: (CHECKOUT_REFUSED[started.error] ?? m.plan_checkout_failed)() });
	},
};
