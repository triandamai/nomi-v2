import { apiUrl } from './api';

// Shipyard posts its events to APP_URL/hooks/shipyard (the URL Nomi registers with it); the
// backend checks the signature over the exact bytes, so the body is passed on untouched, with only
// Shipyard's own headers (never the browser's cookies).
const FORWARDED = ['content-type', 'shipyard-signature', 'shipyard-event-id', 'shipyard-event-type', 'shipyard-delivery-id'];

export async function forwardShipyardHook(request: Request, fetchFn: typeof fetch): Promise<Response> {
	const headers = new Headers();
	for (const name of FORWARDED) {
		const value = request.headers.get(name);
		if (value) headers.set(name, value);
	}
	const response = await fetchFn(apiUrl('/api/hooks/shipyard'), { method: 'POST', headers, body: await request.arrayBuffer() });
	return new Response(null, { status: response.status });
}
