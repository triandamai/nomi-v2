import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

// Shipyard posts its events here (the URL Nomi registers with it); the backend checks the
// signature over the exact bytes, so the body is passed on untouched.
const FORWARDED = ['content-type', 'shipyard-signature', 'shipyard-event-id', 'shipyard-event-type', 'shipyard-delivery-id'];

export const POST: RequestHandler = async ({ request, fetch }) => {
	const headers = new Headers();
	for (const name of FORWARDED) {
		const value = request.headers.get(name);
		if (value) headers.set(name, value);
	}
	const response = await fetch(apiUrl('/api/hooks/shipyard'), { method: 'POST', headers, body: await request.arrayBuffer() });
	return new Response(null, { status: response.status });
};
