import { json } from '@sveltejs/kit';
import { loadPresence } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** Who else has the project open and where, read again whenever someone arrives, moves or leaves. */
export const GET: RequestHandler = async (event) => {
	const presence = await loadPresence(event, event.params.id);
	return presence ? json(presence) : new Response(null, { status: 503 });
};
