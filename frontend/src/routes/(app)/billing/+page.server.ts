import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { BillingPayment, BillingSubscription, Entitlements, PlansForUser, ReferralInfo, UsageMonth } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/** The JSON a backend call answered with, or null when it didn't answer well. */
async function read<T>(response: Response): Promise<T | null> {
	return response.ok ? ((await response.json().catch(() => null)) as T | null) : null;
}

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const month = url.searchParams.get('month');
	const query = month && /^\d{4}-\d{2}$/.test(month) ? `?month=${month}` : '';
	const paymentId = url.searchParams.get('payment');
	const get = (path: string) => apiFetch(fetch, cookies, path);
	const [usageResponse, plansResponse, subscriptionResponse, entitlementsResponse, paymentsResponse, referralResponse, paymentResponse] = await Promise.all([
		get(`/api/usage${query}`),
		get('/api/plans'),
		get('/api/billing/subscription'),
		get('/api/entitlements'),
		get('/api/billing/payments'),
		get('/api/billing/referral'),
		// Coming back from checkout or the invoice: that payment, refreshed from the provider while it's pending.
		paymentId && UUID.test(paymentId) ? get(`/api/billing/payments/${paymentId}`) : Promise.resolve(null),
	]);
	return {
		usage: await read<UsageMonth>(usageResponse),
		plans: await read<PlansForUser>(plansResponse),
		subscription: await read<BillingSubscription>(subscriptionResponse),
		entitlements: await read<Entitlements>(entitlementsResponse),
		payments: (await read<BillingPayment[]>(paymentsResponse)) ?? [],
		referral: await read<ReferralInfo>(referralResponse),
		payment: paymentResponse ? await read<BillingPayment>(paymentResponse) : null,
	};
};

export const actions: Actions = {
	autoRenew: async ({ request, cookies, fetch }) => {
		const on = (await request.formData()).get('auto_renew') === 'true';
		const response = await apiFetch(fetch, cookies, '/api/billing/auto-renew', { method: 'PUT', body: JSON.stringify({ auto_renew: on }) });
		if (!response.ok) return fail(response.status, { error: (await response.text()).trim() || m.billing_action_failed(), action: 'autoRenew' });
		return { autoRenew: on };
	},

	cancel: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		if (typeof id !== 'string' || !UUID.test(id)) return fail(400, { error: m.billing_action_failed(), action: 'cancel' });
		const response = await apiFetch(fetch, cookies, `/api/billing/payments/${id}/cancel`, { method: 'POST' });
		if (!response.ok) return fail(response.status, { error: (await response.text()).trim() || m.billing_action_failed(), action: 'cancel' });
		return { cancelled: id };
	},
};
