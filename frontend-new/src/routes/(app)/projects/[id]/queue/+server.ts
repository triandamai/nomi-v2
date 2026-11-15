import { json } from '@sveltejs/kit';
import { loadQueue } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** The asks in line in the project's chat, read again whenever the chat moves on. */
export const GET: RequestHandler = async (event) => {
	const line = await loadQueue(event, event.params.id);
	return line ? json(line) : new Response(null, { status: 503 });
};
