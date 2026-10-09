import type { RequestHandler } from './$types';
import { apiFetch } from '$lib/server/api';

/** The number on the bell, for $lib/notifications to poll. */
export const GET: RequestHandler = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/notifications/unread');
	const body = response.ok ? await response.text() : JSON.stringify({ unread: 0 });
	return new Response(body, { headers: { 'Content-Type': 'application/json' } });
};
