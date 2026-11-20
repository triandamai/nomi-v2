import { redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

/** The landing page is for people who aren't signed in; the signed-in go to Home. */
export const load: PageServerLoad = ({ locals }) => {
	if (locals.accessToken) redirect(307, '/');
	return {};
};

export const actions: Actions = {
	/**
	 * The hero's composer: what was typed opens a new chat with it in the field (unsent), by way of
	 * sign in for someone who isn't signed in yet (sign in comes back to `redirect_to`).
	 */
	start: async ({ request, locals }) => {
		const text = String((await request.formData()).get('text') ?? '')
			.trim()
			.slice(0, 2000);
		const chat = text ? `/chat/new?q=${encodeURIComponent(text)}` : '/chat/new';
		redirect(303, locals.accessToken ? chat : `/sign-in?redirect_to=${encodeURIComponent(chat)}`);
	},
};
