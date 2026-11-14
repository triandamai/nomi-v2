import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { BillingQuote, BillingTerm, PlansForUser } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, params }) => {
	const [plansResponse, termsResponse] = await Promise.all([apiFetch(fetch, cookies, '/api/plans'), apiFetch(fetch, cookies, '/api/billing/terms')]);
	if (!plansResponse.ok) throw redirect(303, '/billing');
	const offer = (await plansResponse.json()) as PlansForUser;
	const plan = offer.plans.find((p) => p.id === params.planId);
	// Only a plan that's for sale, shown, and not the one they're already on.
	if (!plan || !plan.is_active || plan.price_amount <= 0 || plan.id === offer.current_plan_id) throw redirect(303, '/billing');

	const terms = termsResponse.ok ? ((await termsResponse.json()) as BillingTerm[]).filter((t) => t.is_active).toSorted((a, b) => a.sort_order - b.sort_order || a.months - b.months) : [];
	// What each term costs with no code, so the cards show real prices before anything is chosen.
	const quotes: Record<number, BillingQuote | null> = {};
	await Promise.all(
		terms.map(async (term) => {
			const response = await apiFetch(fetch, cookies, '/api/billing/quote', { method: 'POST', body: JSON.stringify({ plan_id: plan.id, months: term.months }) });
			quotes[term.months] = response.ok ? ((await response.json()) as BillingQuote) : null;
		}),
	);
	return { plan, terms, quotes };
};
