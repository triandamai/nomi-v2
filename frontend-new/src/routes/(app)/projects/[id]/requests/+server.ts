import { json } from '@sveltejs/kit';
import { loadRequests } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** The project's requests to publish, read again whenever one changes. */
export const GET: RequestHandler = async (event) => {
	const rows = await loadRequests(event, event.params.id);
	return rows ? json(rows) : new Response(null, { status: 503 });
};
