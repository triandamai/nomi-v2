import { savePreferences } from '$lib/server/account';
import type { RequestHandler } from './$types';

/** The account menu's theme and language: `PUT { theme } | { language }`, saved with the person's
 * preferences, then kept on this device. 204, or the status that stopped it. */
export const PUT: RequestHandler = async (event) => {
	if (!event.locals.accessToken) return new Response(null, { status: 401 });
	const body = (await event.request.json().catch(() => ({}))) as { theme?: unknown; language?: unknown };
	const result = await savePreferences(event, { theme: body.theme, language: body.language });
	return new Response(null, { status: result.ok ? 204 : result.status });
};
