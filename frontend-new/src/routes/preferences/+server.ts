import { savePreferences, type PreferenceChange } from '$lib/server/account';
import type { RequestHandler } from './$types';

/** The account menu's theme and language, and Settings › General's time zone, who new chats start
 * with and whether Enter sends: `PUT { theme } | { language } | { timezone } | { default_agent } |
 * { enter_sends }`, saved with the person's preferences. 204, or the status that stopped it. */
export const PUT: RequestHandler = async (event) => {
	if (!event.locals.accessToken) return new Response(null, { status: 401 });
	const body = (await event.request.json().catch(() => ({}))) as PreferenceChange;
	const { theme, language, timezone, default_agent, enter_sends } = body ?? {};
	const result = await savePreferences(event, { theme, language, timezone, default_agent, enter_sends });
	return new Response(null, { status: result.ok ? 204 : result.status });
};
