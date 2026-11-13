import type { AdminPlan } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import { m } from '$lib/paraglide/messages';
import type { Actions, PageServerLoad } from './$types';

/** The plans (plans: view): what each gives, its price, who is on it, which is the default. */
export const load: PageServerLoad = async (event) => ({ plans: mustLoad(await getJson<AdminPlan[]>(event, '/api/admin/plans')) });

export const actions: Actions = {
	/** The plan new people start on (plans: write). */
	makeDefault: async (event) => {
		const id = field(await event.request.formData(), 'id');
		if (!id) return refuse('makeDefault', m.admin_plan_required());
		return act(event, 'makeDefault', `/api/admin/plans/${encodeURIComponent(id)}/default`, 'PUT');
	},
};
