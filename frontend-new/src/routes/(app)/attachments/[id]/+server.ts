import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// What the backend knows about one of the person's files (its pages, for the viewer), and removing
// one taken out of the composer before it was sent.
export const GET: RequestHandler = async (event) => {
	const response = await apiFetch(event, `/api/attachments/${encodeURIComponent(event.params.id)}`, undefined, { signedOut: 'return' });
	return new Response(response.body, { status: response.status, headers: { 'content-type': 'application/json' } });
};

export const DELETE: RequestHandler = async (event) => {
	const response = await apiFetch(event, `/api/attachments/${encodeURIComponent(event.params.id)}`, { method: 'DELETE' }, { signedOut: 'return' });
	return new Response(null, { status: response.status });
};
