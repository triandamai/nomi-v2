import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

/**
 * On a phone, Settings is the You tab (Flow / Mobile — Home, Inbox & You, step 3 O1SXw): who's signed
 * in, the plan, and every settings page in its group. Elsewhere it opens on Profile, the first of its
 * pages (App / Settings / Profile U9Gnrg).
 */
export const load: PageServerLoad = async ({ parent }) => {
	const { phone } = await parent();
	if (!phone) redirect(307, '/settings/profile');
	return {};
};
