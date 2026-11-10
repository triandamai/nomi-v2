import { sequence } from '@sveltejs/kit/hooks';
import { redirect, type Handle, type HandleFetch } from '@sveltejs/kit';
import { building } from '$app/environment';
import { paraglideMiddleware } from '$lib/paraglide/server';
import { DEVICE_COOKIE } from '$lib/device';
import { authenticate, isApiRequest } from '$lib/server/api';
import { clientHeaders } from '$lib/server/clientHeaders';
import { legacyPath } from '$lib/server/compat';
import { ensureDeviceId } from '$lib/server/device';
import { THEME_COOKIE, parseTheme, withTheme } from '$lib/theme';

/** Renders each request in the person's language and theme, so the first frame is already right. */
const languageAndTheme: Handle = ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		event.request = request;
		return resolve(event, {
			transformPageChunk: ({ html }) =>
				withTheme(html.replace('%paraglide.lang%', locale), parseTheme(event.cookies.get(THEME_COOKIE))),
		});
	});

/** The backend links to the old app's addresses in emails and notifications: each has a v2 page. */
const legacyAddresses: Handle = ({ event, resolve }) => {
	const to = legacyPath(event.url);
	if (to) redirect(307, to);
	return resolve(event);
};

/** Every browser is a device with an id of its own, from its first visit (before it signs in). */
const device: Handle = ({ event, resolve }) => {
	if (!building) ensureDeviceId(event.cookies);
	return resolve(event);
};

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

export const handle = sequence(legacyAddresses, languageAndTheme, device, auth, noStalePages);

/** Every call to the backend says which version of the frontend made it, and from which browser. */
export const handleFetch: HandleFetch = ({ event, request, fetch }) => {
	if (isApiRequest(request.url)) request = new Request(request, { headers: clientHeaders(request.headers, event.cookies.get(DEVICE_COOKIE)) });
	return fetch(request);
};
