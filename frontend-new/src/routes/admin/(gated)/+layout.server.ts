import { error, redirect } from '@sveltejs/kit';
import { NO_ADMIN_ACCESS, areaForPath, can, homeFor } from '$lib/admin/access';
import type { LayoutServerLoad } from './$types';

/**
 * Who may open which page: nobody without an admin area (the console's no-access page, a 403), and
 * a page in an area someone doesn't have sends them to the first page they can open. The backend's
 * require_admin is still the real check on everything the pages load and change.
 */
export const load: LayoutServerLoad = async ({ parent, url }) => {
	const { access } = await parent();
	const home = homeFor(access);
	if (!home) error(403, { message: NO_ADMIN_ACCESS });
	const area = areaForPath(url.pathname);
	if (area && !can(access, area)) redirect(303, home);
	return {};
};
