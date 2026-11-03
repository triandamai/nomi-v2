import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// The person's GitHub connection, for pushing a project there (Build → Export): whether it's
// connected and as whom (GET), connecting with a token (PUT), or starting GitHub's own sign-in
// (POST, with where to come back to afterwards).

export const GET: RequestHandler = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/connections/github');
	if (!response.ok) throw error(response.status, 'could not read the GitHub connection');
	return json(await response.json());
};

export const PUT: RequestHandler = async ({ request, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/connections/github', { method: 'PUT', body: await request.text() });
	return new Response(response.body, { status: response.status, headers: { 'content-type': response.headers.get('content-type') ?? 'application/json' } });
};

export const POST: RequestHandler = async ({ request, url, cookies, fetch }) => {
	const back = ((await request.json().catch(() => ({}))) as { back?: string }).back;
	if (back && back.startsWith('/') && !back.startsWith('//'))
		cookies.set('nomi_github_back', back, { path: '/', httpOnly: true, sameSite: 'lax', secure: url.protocol === 'https:', maxAge: 600 });
	const response = await apiFetch(fetch, cookies, '/api/connections/github/start', { method: 'POST' });
	return new Response(response.body, { status: response.status, headers: { 'content-type': response.headers.get('content-type') ?? 'application/json' } });
};
