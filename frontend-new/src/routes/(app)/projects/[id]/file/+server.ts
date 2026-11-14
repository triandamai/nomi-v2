import { json } from '@sveltejs/kit';
import { projectFile } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** One of the project's files (`?path=`), for the code view, read only. */
export const GET: RequestHandler = async (event) => {
	const path = event.url.searchParams.get('path') ?? '';
	if (!path || path.includes('..')) return json({ error: 'path', status: 400 }, { status: 400 });
	const file = await projectFile(event, event.params.id, path);
	return 'error' in file ? json(file, { status: file.status }) : json(file);
};
