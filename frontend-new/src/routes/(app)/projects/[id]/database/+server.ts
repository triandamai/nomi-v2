import { json } from '@sveltejs/kit';
import { readDatabase } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** The Database sheet: the tables and the first rows of `?table=` (schema.name), from `?source=`. */
export const GET: RequestHandler = async (event) => {
	const source = event.url.searchParams.get('source') === 'production' ? 'production' : 'development';
	const view = await readDatabase(event, event.params.id, source, event.url.searchParams.get('table'));
	return 'error' in view ? json(view, { status: view.status }) : json(view);
};
