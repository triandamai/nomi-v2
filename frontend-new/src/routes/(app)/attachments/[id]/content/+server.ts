import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// The file itself, to its owner only, as the backend serves it: shown in the page only when it's a
// type a browser can't run (images, PDF, audio, video), downloaded otherwise; ranges for seeking.
const KEPT = ['content-type', 'content-disposition', 'content-length', 'content-range', 'accept-ranges', 'cache-control', 'x-content-type-options'];

export const GET: RequestHandler = async (event) => {
	const range = event.request.headers.get('range');
	const response = await apiFetch(event, `/api/attachments/${encodeURIComponent(event.params.id)}/content`, range ? { headers: { range } } : undefined, { signedOut: 'return' });
	const headers = new Headers();
	for (const name of KEPT) {
		const value = response.headers.get(name);
		if (value) headers.set(name, value);
	}
	if (event.url.searchParams.has('download')) {
		const disposition = headers.get('content-disposition');
		if (disposition) headers.set('content-disposition', disposition.replace(/^inline/, 'attachment'));
	}
	return new Response(response.body, { status: response.status, headers });
};
