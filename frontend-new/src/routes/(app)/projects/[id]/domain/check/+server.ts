import { json } from '@sveltejs/kit';
import { checkDomain } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** Whether a name is free (`?name=`), as the Domain sheet's field asks while it's typed. */
export const GET: RequestHandler = async (event) => {
	const name = event.url.searchParams.get('name')?.trim() ?? '';
	if (!name) return json({ error: 'name' }, { status: 400 });
	return json(await checkDomain(event, event.params.id, name));
};
