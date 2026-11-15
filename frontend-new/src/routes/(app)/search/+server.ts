import { json } from '@sveltejs/kit';
import { searchAll } from '$lib/server/search';
import type { RequestHandler } from './$types';

/** The top bar's search, as it's typed (`?q=`). */
export const GET: RequestHandler = async (event) => {
	const results = await searchAll(event, event.url.searchParams.get('q') ?? '');
	return results ? json({ results }) : json({ error: 'search_failed' }, { status: 503 });
};
