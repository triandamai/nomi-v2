import { json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// What deleting a message would take with it (GET), and deleting it (DELETE).
async function forward(fetch: typeof globalThis.fetch, cookies: Parameters<typeof apiFetch>[1], path: string, method: 'GET' | 'DELETE') {
	const response = await apiFetch(fetch, cookies, path, { method });
	const body = response.ok ? await response.json() : { error: await response.text() };
	return json(body, { status: response.status });
}

export const GET: RequestHandler = ({ params, cookies, fetch }) =>
	forward(fetch, cookies, `/api/sessions/${params.sessionId}/messages/${params.messageId}/delete`, 'GET');

export const DELETE: RequestHandler = ({ params, cookies, fetch }) =>
	forward(fetch, cookies, `/api/sessions/${params.sessionId}/messages/${params.messageId}`, 'DELETE');
