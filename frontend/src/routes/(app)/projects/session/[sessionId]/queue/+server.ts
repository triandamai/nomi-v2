import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

/** The asks waiting for the crew in this chat, in the order they'll be answered. */
export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/queue`);
	if (!response.ok) throw error(response.status, 'could not load the queue');
	return json(await response.json());
};
