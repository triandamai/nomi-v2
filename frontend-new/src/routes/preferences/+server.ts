import { savePreferences } from '$lib/server/account';
import type { RequestHandler } from './$types';

/** The account menu's theme and language, and Settings › General's time zone: `PUT { theme } |
 * { language } | { timezone }`, saved with the person's preferences. 204, or the status that stopped it. */
export const PUT: RequestHandler = async (event) => {
	if (!event.locals.accessToken) return new Response(null, { status: 401 });
	const body = (await event.request.json().catch(() => ({}))) as { theme?: unknown; language?: unknown; timezone?: unknown };
	const result = await savePreferences(event, { theme: body.theme, language: body.language, timezone: body.timezone });
	return new Response(null, { status: result.ok ? 204 : result.status });
};
