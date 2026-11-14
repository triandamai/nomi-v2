import { json } from '@sveltejs/kit';
import { loadDeploy } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** The deploy overview, which the workspace looks at again while a publish runs. */
export const GET: RequestHandler = async (event) => {
	const overview = await loadDeploy(event, event.params.id);
	return overview ? json(overview) : new Response(null, { status: 503 });
};
