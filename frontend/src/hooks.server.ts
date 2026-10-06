import { sequence } from '@sveltejs/kit/hooks';
import type { Handle } from '@sveltejs/kit';
import { paraglideMiddleware } from '$lib/paraglide/server';

/** Renders each request in the person's language and sets `<html lang>` to match. */
const i18n: Handle = ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		event.request = request;
		return resolve(event, {
			transformPageChunk: ({ html }) => html.replace('%paraglide.lang%', locale),
		});
	});

const auth: Handle = async ({ event, resolve }) => {
	event.locals.accessToken = event.cookies.get('access_token');
	return resolve(event);
};

export const handle = sequence(i18n, auth);
