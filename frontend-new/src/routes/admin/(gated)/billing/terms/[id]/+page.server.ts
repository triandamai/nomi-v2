import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { BillingTerm } from '$lib/admin/api';
import { termFromForm } from '$lib/admin/billing';
import { act, getJson, mustLoad, refuse, send } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/** A term to add (`/admin/billing/terms/new`) or change (plans: write). An existing one comes from the list. */
export const load: PageServerLoad = async (event) => {
	if (event.params.id === 'new') return { title: m.admin_term_new(), term: null };
	const list = mustLoad(await getJson<BillingTerm[]>(event, '/api/admin/billing/terms'));
	if (!list.ok) return { title: m.admin_page_terms(), term: list };
	const term = list.data.find((one) => one.id === event.params.id);
	if (!term) error(404, { message: m.admin_term_missing() });
	return { title: m.admin_term_months({ count: String(term.months) }), term: { ok: true as const, data: term } };
};

export const actions: Actions = {
	save: async (event) => {
		const read = termFromForm(await event.request.formData());
		if (!read.ok) return refuse('save', read.message);
		if (event.params.id !== 'new') return act(event, 'save', `/api/admin/billing/terms/${encodeURIComponent(event.params.id)}`, 'PUT', read.input);
		const created = await send(event, '/api/admin/billing/terms', 'POST', read.input);
		if (!created.ok) return fail(created.problem.status >= 400 ? created.problem.status : 500, { action: 'save', ok: false as const, problem: created.problem });
		redirect(303, '/admin/billing/terms');
	},

	delete: async (event) => {
		const deleted = await act(event, 'delete', `/api/admin/billing/terms/${encodeURIComponent(event.params.id)}`, 'DELETE');
		if ('ok' in deleted && deleted.ok) redirect(303, '/admin/billing/terms');
		return deleted;
	},
};
