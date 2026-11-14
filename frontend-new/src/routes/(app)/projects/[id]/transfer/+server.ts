import { json } from '@sveltejs/kit';
import { transferTargets } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** The workspaces the project can move to (`?q=` narrows them by name). */
export const GET: RequestHandler = async (event) => {
	const targets = await transferTargets(event, event.params.id, event.url.searchParams.get('q')?.trim() ?? '');
	return targets ? json(targets) : new Response(null, { status: 503 });
};
