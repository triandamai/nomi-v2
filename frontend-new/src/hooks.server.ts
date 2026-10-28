import type { Handle } from '@sveltejs/kit';
import { paraglideMiddleware } from '$lib/paraglide/server';
import { THEME_COOKIE, parseTheme, withTheme } from '$lib/theme';

/** Renders each request in the person's language and theme, so the first frame is already right. */
export const handle: Handle = ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		event.request = request;
		return resolve(event, {
			transformPageChunk: ({ html }) =>
				withTheme(html.replace('%paraglide.lang%', locale), parseTheme(event.cookies.get(THEME_COOKIE))),
		});
	});
