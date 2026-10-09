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

/**
 * The project page runs projects in WebContainer, which needs the page cross-origin isolated.
 * Only that page gets these headers: elsewhere they'd get in the way of Google sign-in popups.
 * `credentialless` still lets the page load cross-origin images and fonts (without cookies).
 */
const crossOriginIsolation: Handle = async ({ event, resolve }) => {
	const response = await resolve(event);
	if (event.url.pathname.startsWith('/projects/session/') && response.headers.get('content-type')?.startsWith('text/html')) {
		response.headers.set('Cross-Origin-Opener-Policy', 'same-origin');
		response.headers.set('Cross-Origin-Embedder-Policy', 'credentialless');
	}
	// The static preview is framed (sandboxed, so cross-origin) inside that isolated page, so it
	// has to opt in too.
	if (/^\/projects\/[^/]+\/preview(\/|$)/.test(event.url.pathname)) {
		response.headers.set('Cross-Origin-Embedder-Policy', 'credentialless');
		response.headers.set('Cross-Origin-Resource-Policy', 'cross-origin');
	}
	return response;
};

export const handle = sequence(i18n, auth, crossOriginIsolation);

/** Every call to the backend says which version of the frontend made it. */
export const handleFetch: HandleFetch = ({ request, fetch }) => {
	if (isApiRequest(request.url)) {
		const headers = new Headers(request.headers);
		headers.set(CLIENT_VERSION_HEADER, APP_VERSION);
		request = new Request(request, { headers });
	}
	return fetch(request);
};
