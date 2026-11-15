import { apiUrl } from './api';

// A published site reports each visit's referrer and each form sent to APP_URL/hooks/sites/:id
// (the script a publish puts in its pages). In production server.js answers it first
// (ws-proxy/site-report-proxy.js), since SvelteKit reads no body without a content type and the
// script sends none; this serves `vite dev`/`preview` and a report that names its type. The visitor
// isn't signed in to Nomi: it goes on as it came, and the answer is always empty, to any site.
const MAX_REPORT_BYTES = 2048;

export async function forwardSiteReport(projectId: string, request: Request, fetchFn: typeof fetch): Promise<Response> {
	const body = await request.arrayBuffer();
	if (body.byteLength <= MAX_REPORT_BYTES) {
		await fetchFn(apiUrl(`/api/hooks/sites/${encodeURIComponent(projectId)}`), { method: 'POST', body }).catch(() => undefined);
	}
	return new Response(null, { status: 204, headers: { 'access-control-allow-origin': '*' } });
}
