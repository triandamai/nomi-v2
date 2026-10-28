import { sequence } from '@sveltejs/kit/hooks';
import type { Handle, HandleFetch } from '@sveltejs/kit';
import { paraglideMiddleware } from '$lib/paraglide/server';
import { authenticate, isApiRequest } from '$lib/server/api';
import { THEME_COOKIE, parseTheme, withTheme } from '$lib/theme';
import { APP_VERSION, CLIENT_VERSION_HEADER } from '$lib/version';

/** Renders each request in the person's language and theme, so the first frame is already right. */
const languageAndTheme: Handle = ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		event.request = request;
		return resolve(event, {
			transformPageChunk: ({ html }) =>
				withTheme(html.replace('%paraglide.lang%', locale), parseTheme(event.cookies.get(THEME_COOKIE))),
		});
	});

/** The person's sign-in for this request, renewed first when it's about to expire. */
const auth: Handle = async ({ event, resolve }) => {
	if (!event.url.pathname.startsWith('/sign-out')) await authenticate(event);
	return resolve(event);
};

/**
 * Pages are revalidated on every load. The HTML names this build's app files, which go away with
 * the next deploy; a page kept in a cache past that can't start.
 */
const noStalePages: Handle = async ({ event, resolve }) => {
	const response = await resolve(event);
	if (response.headers.get('content-type')?.startsWith('text/html') && !response.headers.has('cache-control')) {
		response.headers.set('cache-control', 'no-cache');
	}
	return response;
};

export const handle = sequence(languageAndTheme, auth, noStalePages);

/** Every call to the backend says which version of the frontend made it. */
export const handleFetch: HandleFetch = ({ request, fetch }) => {
	if (isApiRequest(request.url)) {
		const headers = new Headers(request.headers);
		headers.set(CLIENT_VERSION_HEADER, APP_VERSION);
		request = new Request(request, { headers });
	}
	return fetch(request);
};
