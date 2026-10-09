import type { RequestHandler } from './$types';
import { apiFetch } from '$lib/server/api';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/attachments/${params.id}`);
	return new Response(response.body, { status: response.status, headers: { 'Content-Type': 'application/json' } });
};

export const DELETE: RequestHandler = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/attachments/${params.id}`, { method: 'DELETE' });
	return new Response(null, { status: response.status });
};
