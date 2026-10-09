import type { RequestEvent } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';

/** Request headers an upload carries through to the backend. */
export const UPLOAD_HEADERS = ['content-type', 'content-length', 'x-file-name', 'x-attachment-kind', 'x-transcript'] as const;

/** Response headers a file keeps on its way back to the browser. */
const FILE_HEADERS = ['content-type', 'content-length', 'content-disposition', 'content-range', 'accept-ranges', 'cache-control', 'x-content-type-options'];

/** Streams a file from the backend to the browser, passing a video's Range request along. */
export async function relayFile(event: RequestEvent, path: string): Promise<Response> {
	const range = event.request.headers.get('range');
	const response = await apiFetch(event.fetch, event.cookies, path, { headers: range ? { Range: range } : {} });
	const headers = new Headers();
	for (const name of FILE_HEADERS) {
		const value = response.headers.get(name);
		if (value) headers.set(name, value);
	}
	return new Response(response.body, { status: response.status, headers });
}
