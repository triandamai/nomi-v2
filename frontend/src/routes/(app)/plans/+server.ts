import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { apiFetch } from '$lib/server/api';

/** The plans on offer and the person's own (GET /api/plans), for the plans sheet. */
export const GET: RequestHandler = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/plans');
	if (!response.ok) return json(null, { status: response.status });
	return json(await response.json());
};
