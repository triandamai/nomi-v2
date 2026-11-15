import { json } from '@sveltejs/kit';
import { projectAbout } from '$lib/server/projects';
import type { RequestHandler } from './$types';

/** What the Projects page's detail panel (and the Deploy dialog) says about a project. */
export const GET: RequestHandler = async (event) => {
	const about = await projectAbout(event, event.params.id);
	return about ? json(about) : new Response(null, { status: 404 });
};
