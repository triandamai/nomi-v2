import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

/** A project's files as a zip, passed through from the backend as it streams. */
export const GET: RequestHandler = async (event) => {
	const response = await apiFetch(event, `/api/projects/${encodeURIComponent(event.params.id)}/export/zip`);
	if (!response.ok || !response.body) error(response.status === 404 ? 404 : 502, { message: 'export failed' });
	const headers = new Headers({ 'content-type': response.headers.get('content-type') ?? 'application/zip' });
	const disposition = response.headers.get('content-disposition');
	headers.set('content-disposition', disposition ?? `attachment; filename="project-${event.params.id}.zip"`);
	return new Response(response.body, { headers });
};
