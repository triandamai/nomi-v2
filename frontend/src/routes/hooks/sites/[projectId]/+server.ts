import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

// A published site reports its visits and forms sent here (the script Nomi puts in its pages on
// publish). The visitor's browser isn't signed in to Nomi: the body goes on as it came, and the
// answer is always empty.
const MAX_REPORT_BYTES = 2048;

export const POST: RequestHandler = async ({ params, request, fetch }) => {
	const body = await request.arrayBuffer();
	if (body.byteLength <= MAX_REPORT_BYTES) {
		await fetch(apiUrl(`/api/hooks/sites/${encodeURIComponent(params.projectId)}`), { method: 'POST', body }).catch(() => undefined);
	}
	return new Response(null, { status: 204, headers: { 'access-control-allow-origin': '*' } });
};
