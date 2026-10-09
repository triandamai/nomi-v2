import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// The open project page runs the project in the browser and talks to the backend through
// here: checking in for changed files and commands, posting results, and loading files.
const ALLOWED = /^(runtime|runtime\/check|runtime\/runs\/[0-9a-f-]{36}|snapshot|manifest|files\/.+)$/;

const proxy: RequestHandler = async ({ params, request, url, cookies, fetch }) => {
	if (!ALLOWED.test(params.rest)) throw error(404, 'Not found');
	const path = params.rest
		.split('/')
		.map((part) => encodeURIComponent(part))
		.join('/');
	const init: RequestInit = { method: request.method, signal: request.signal };
	if (request.method !== 'GET' && request.method !== 'HEAD') init.body = await request.text();
	const response = await apiFetch(fetch, cookies, `/api/projects/${encodeURIComponent(params.projectId)}/${path}${url.search}`, init);
	return new Response(response.body, {
		status: response.status,
		headers: { 'content-type': response.headers.get('content-type') ?? 'application/json', 'cache-control': 'no-store' },
	});
};

export const GET = proxy;
export const POST = proxy;
