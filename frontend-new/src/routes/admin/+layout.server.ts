import { isRedirect, redirect } from '@sveltejs/kit';
import { SIGN_IN_PATH } from '$lib/server/api';
import { loadAdminShell } from '$lib/server/adminShell';
import type { LayoutServerLoad } from './$types';

/**
 * The admin console around every admin page: someone signed out signs in and comes back here;
 * everyone else gets their admin access (from their token) and the shell's data. Maintenance
 * never closes the console: it is where maintenance is turned off. `invalidate('admin:shell')`
 * reloads it.
 */
export const load: LayoutServerLoad = async (event) => {
	event.depends('admin:shell');
	const here = event.untrack(() => event.url.pathname + event.url.search);
	const signIn = `${SIGN_IN_PATH}?redirect_to=${encodeURIComponent(here)}`;
	if (!event.locals.accessToken) redirect(303, signIn);

	try {
		return await loadAdminShell(event);
	} catch (error) {
		if (isRedirect(error) && error.location === SIGN_IN_PATH) redirect(303, signIn);
		throw error;
	}
};
