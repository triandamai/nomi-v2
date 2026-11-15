import { json } from '@sveltejs/kit';
import { moveFirst } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** Lets this waiting ask go first: the line as it is now, or the backend's reason it couldn't. */
export const POST: RequestHandler = async (event) => {
	const moved = await moveFirst(event, event.params.id, event.params.job);
	return moved.ok ? json({ asks: moved.asks }) : json({ message: moved.message }, { status: moved.status });
};
