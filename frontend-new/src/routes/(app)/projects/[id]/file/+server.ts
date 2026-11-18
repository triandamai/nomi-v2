import { json } from '@sveltejs/kit';
import { projectFile } from '$lib/server/workspace';
import { highlightLines } from '$lib/server/codeTokens';
import type { RequestHandler } from './$types';

/**
 * One of the project's files (`?path=`), for the code view, read only. A text file comes with its
 * lines already highlighted (`lines`), so no highlighter ships to the browser.
 */
export const GET: RequestHandler = async (event) => {
	const path = event.url.searchParams.get('path') ?? '';
	if (!path || path.includes('..')) return json({ error: 'path', status: 400 }, { status: 400 });
	const file = await projectFile(event, event.params.id, path);
	if ('error' in file) return json(file, { status: file.status });
	if (file.binary || typeof file.content !== 'string') return json(file);
	return json({ ...file, lines: await highlightLines(file.content, path) });
};
