import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

/** Answers this waiting ask next. */
export const POST: RequestHandler = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/queue/${params.jobId}/first`, { method: 'POST' });
	if (!response.ok) throw error(response.status, 'could not move the ask');
	return json(await response.json());
};
