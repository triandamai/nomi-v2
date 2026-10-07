import { sequence } from '@sveltejs/kit/hooks';
import type { Handle, HandleFetch } from '@sveltejs/kit';
import { isApiRequest } from '$lib/server/api';
import { APP_VERSION, CLIENT_VERSION_HEADER } from '$lib/version';
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

/** Every call to the backend says which version of the frontend made it. */
export const handleFetch: HandleFetch = ({ request, fetch }) => {
	if (isApiRequest(request.url)) {
		const headers = new Headers(request.headers);
		headers.set(CLIENT_VERSION_HEADER, APP_VERSION);
		request = new Request(request, { headers });
	}
	return fetch(request);
};
