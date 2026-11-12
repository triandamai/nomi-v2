import { error } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { loadPlan } from '$lib/server/plan';
import type { PageServerLoad } from './$types';

/** Settings › Plan & usage (App / Settings / Plan & usage C0og49); `?month=YYYY-MM` for another month. */
export const load: PageServerLoad = async (event) => {
	const data = await loadPlan(event, event.url.searchParams.get('month'));
	if (!data) error(503, { message: m.usage_load_failed() });
	// The top bar's Export usage CSV, for the month shown.
	return { title: m.settings_plan(), csv: `/settings/plan/usage.csv?month=${data.usage.month}`, ...data };
};
