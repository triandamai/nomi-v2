import { json } from '@sveltejs/kit';
import { exportSummary } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** What an export holds, for the Export dialog. */
export const GET: RequestHandler = async (event) => {
	const summary = await exportSummary(event, event.params.id);
	return summary ? json(summary) : new Response(null, { status: 503 });
};
