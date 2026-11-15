import { json } from '@sveltejs/kit';
import { moveChoices } from '$lib/server/projects';
import type { RequestHandler } from './$types';

/** The workspace a project is in and those it can move to, for the Projects page's Move dialog. */
export const GET: RequestHandler = async (event) => {
	const choices = await moveChoices(event, event.params.id);
	return choices ? json(choices) : new Response(null, { status: 503 });
};
