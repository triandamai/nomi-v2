import { fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { PlansForUser, UsageMonth } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const month = url.searchParams.get('month');
	const query = month && /^\d{4}-\d{2}$/.test(month) ? `?month=${month}` : '';
	const [usageResponse, plansResponse] = await Promise.all([apiFetch(fetch, cookies, `/api/usage${query}`), apiFetch(fetch, cookies, '/api/plans')]);
	const usage: UsageMonth | null = usageResponse.ok ? ((await usageResponse.json()) as UsageMonth) : null;
	const plans: PlansForUser | null = plansResponse.ok ? ((await plansResponse.json()) as PlansForUser) : null;
	return { usage, plans };
};

const UUID = /^[0-9a-f-]{36}$/i;

function planId(data: FormData): string | null {
	const id = data.get('plan_id');
	return typeof id === 'string' && UUID.test(id) ? id : null;
}

async function errorCode(response: Response): Promise<string> {
	try {
		const body = (await response.json()) as { error?: unknown };
		return typeof body.error === 'string' ? body.error : 'failed';
	} catch {
		return 'failed';
	}
}

export const actions: Actions = {
	/** Starts paying for a plan: off to the payment gateway's page. */
	checkout: async ({ request, cookies, fetch }) => {
		const id = planId(await request.formData());
		if (!id) return fail(400, { checkoutError: 'failed', planId: null });
		const response = await apiFetch(fetch, cookies, `/api/plans/${id}/checkout`, { method: 'POST' });
		if (!response.ok) return fail(response.status, { checkoutError: await errorCode(response), planId: id });
		const { url } = (await response.json()) as { url: string };
		redirect(303, url);
	},

	/** Asks to be told when a plan opens. */
	waitlist: async ({ request, cookies, fetch }) => {
		const id = planId(await request.formData());
		if (!id) return fail(400, { waitlistError: true, planId: null });
		const response = await apiFetch(fetch, cookies, `/api/plans/${id}/waitlist`, { method: 'POST' });
		if (!response.ok) return fail(response.status, { waitlistError: true, planId: id });
		return { waitlisted: id };
	},
};
