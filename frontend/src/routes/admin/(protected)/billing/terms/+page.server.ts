import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { BillingTerm } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';
import { failure } from '../failure';
import { readTerm } from './termForm';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/billing/terms');
	const terms: BillingTerm[] = response.ok ? ((await response.json()) as BillingTerm[]) : [];
	return { terms: terms.toSorted((a, b) => a.sort_order - b.sort_order || a.months - b.months) };
};

export const actions: Actions = {
	save: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const term = readTerm(data);
		if (!term) return fail(400, { error: m.billing_admin_term_invalid() });
		const id = data.get('id');
		const editing = typeof id === 'string' && id.length > 0;
		const response = await apiFetch(fetch, cookies, editing ? `/api/admin/billing/terms/${id}` : '/api/admin/billing/terms', {
			method: editing ? 'PUT' : 'POST',
			body: JSON.stringify(term),
		});
		return response.ok ? { saved: true } : failure(response, m.billing_admin_save_failed());
	},

	delete: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		const response = await apiFetch(fetch, cookies, `/api/admin/billing/terms/${id}`, { method: 'DELETE' });
		return response.ok ? { success: true } : failure(response, m.billing_admin_delete_failed());
	},
};
