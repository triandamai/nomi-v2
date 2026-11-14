import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { AdminPlan } from '$lib/admin/api';
import { planFromForm } from '$lib/admin/plans';
import { act, getJson, mustLoad, refuse, send } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * A plan to add (`/admin/plans/new`) or change (plans: write; view shows it). There's no
 * single-plan endpoint, so an existing one comes from the list.
 */
export const load: PageServerLoad = async (event) => {
	if (event.params.id === 'new') return { title: m.admin_plan_new(), plan: null };
	const list = mustLoad(await getJson<AdminPlan[]>(event, '/api/admin/plans'));
	if (!list.ok) return { title: m.admin_page_plans(), plan: list };
	const plan = list.data.find((one) => one.id === event.params.id);
	if (!plan) error(404, { message: m.admin_plan_missing() });
	return { title: plan.name, plan: { ok: true as const, data: plan } };
};

export const actions: Actions = {
	save: async (event) => {
		const read = planFromForm(await event.request.formData());
		if (!read.ok) return refuse('save', read.message);
		if (event.params.id !== 'new') return act(event, 'save', `/api/admin/plans/${encodeURIComponent(event.params.id)}`, 'PUT', read.input);
		// A new plan opens as itself once it's made.
		const created = await send<{ id: string }>(event, '/api/admin/plans', 'POST', read.input);
		if (!created.ok) return fail(created.problem.status >= 400 ? created.problem.status : 500, { action: 'save', ok: false as const, problem: created.problem });
		redirect(303, `/admin/plans/${created.data.id}`);
	},

	delete: async (event) => {
		const deleted = await act(event, 'delete', `/api/admin/plans/${encodeURIComponent(event.params.id)}`, 'DELETE');
		if ('ok' in deleted && deleted.ok) redirect(303, '/admin/plans');
		return deleted;
	},
};
