import { error } from '@sveltejs/kit';
import { loadMemories } from '$lib/server/memory';
import type { RequestHandler } from './$types';

/** Memory & context's Export: everything the crew remembers, as plain JSON to keep. */
export const GET: RequestHandler = async (event) => {
	const memories = await loadMemories(event);
	if (!memories) error(503, 'memory unavailable');
	return new Response(JSON.stringify({ memories }, null, 2), {
		headers: { 'content-type': 'application/json; charset=utf-8', 'content-disposition': 'attachment; filename="nomi-memory.json"' },
	});
};
