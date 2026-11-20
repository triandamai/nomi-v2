import { redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import { appHref, domainsFrom, landingHome } from '$lib/domains';
import type { Actions, PageServerLoad } from './$types';

/**
 * The landing page is for people who aren't signed in; the signed-in go to Home. On its own
 * domain (PUBLIC_LANDING_HOST) its links to the app go to the app's domain.
 */
export const load: PageServerLoad = ({ locals, url }) => {
	if (locals.accessToken) redirect(307, '/');
	const domains = domainsFrom(env);
	return { home: landingHome(url.hostname, domains), signIn: appHref('/sign-in', url, domains), register: appHref('/register', url, domains) };
};

export const actions: Actions = {
	/**
	 * The hero's composer: what was typed opens a new chat with it in the field (unsent), by way of
	 * sign in for someone who isn't signed in yet (sign in comes back to `redirect_to`).
	 */
	start: async ({ request, locals, url }) => {
		const text = String((await request.formData()).get('text') ?? '')
			.trim()
			.slice(0, 2000);
		const chat = text ? `/chat/new?q=${encodeURIComponent(text)}` : '/chat/new';
		redirect(303, appHref(locals.accessToken ? chat : `/sign-in?redirect_to=${encodeURIComponent(chat)}`, url, domainsFrom(env)));
	},
};
