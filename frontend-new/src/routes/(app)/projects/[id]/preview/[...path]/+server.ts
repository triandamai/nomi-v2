import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

/**
 * The project's own files, as the workspace's preview frame loads them (index.html, then what it
 * asks for, relative to here), passed through from the backend with the person's sign-in.
 */
export const GET: RequestHandler = async (event) => {
	const id = encodeURIComponent(event.params.id);
	const path = event.params.path.split('/').map(encodeURIComponent).join('/');
	const response = await apiFetch(event, path ? `/api/projects/${id}/preview/${path}` : `/api/projects/${id}/preview`);
	const headers = new Headers({ 'content-type': response.headers.get('content-type') ?? 'application/octet-stream', 'cache-control': 'no-store' });
	return new Response(response.body, { status: response.status, headers });
};
