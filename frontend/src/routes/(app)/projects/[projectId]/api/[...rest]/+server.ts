import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { ALLOWED_PROJECT_API_PATHS } from '$lib/projectTransfer';
import type { RequestHandler } from './$types';

// The open project page talks to the backend through here: the in-browser runtime (changed
// files, commands, results, files), and the project's sheets (issues, publishing, monitoring,
// database, members, settings). An empty path is the project itself (deleting it).
const proxy: RequestHandler = async ({ params, request, url, cookies, fetch }) => {
	if (!ALLOWED_PROJECT_API_PATHS.test(params.rest)) throw error(404, 'Not found');
	const path = params.rest
		.split('/')
		.map((part) => encodeURIComponent(part))
		.join('/');
	const init: RequestInit = { method: request.method, signal: request.signal };
	if (request.method !== 'GET' && request.method !== 'HEAD') init.body = await request.text();
	const target = `/api/projects/${encodeURIComponent(params.projectId)}${path ? `/${path}` : ''}${url.search}`;
	const response = await apiFetch(fetch, cookies, target, init);
	return new Response(response.body, {
		status: response.status,
		headers: { 'content-type': response.headers.get('content-type') ?? 'application/json', 'cache-control': 'no-store' },
	});
};

export const GET = proxy;
export const POST = proxy;
export const PUT = proxy;
export const PATCH = proxy;
export const DELETE = proxy;
