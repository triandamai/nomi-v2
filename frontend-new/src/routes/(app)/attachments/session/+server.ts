import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// Renews the sign-in for an upload that came back 401 (uploads go around SvelteKit, so nothing
// renewed it on the way): any call through apiFetch does, and sets the new cookies.
export const POST: RequestHandler = async (event) => {
	const response = await apiFetch(event, '/api/attachments/limits', undefined, { signedOut: 'return' });
	return new Response(null, { status: response.ok ? 204 : response.status });
};
