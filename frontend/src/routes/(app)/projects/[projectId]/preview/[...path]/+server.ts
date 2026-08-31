import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const suffix = params.path ? `/${params.path}` : '';
	const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}/preview${suffix}`);
	const contentType = response.headers.get('content-type') ?? 'text/plain';
	return new Response(response.body, { status: response.status, headers: { 'content-type': contentType } });
};
