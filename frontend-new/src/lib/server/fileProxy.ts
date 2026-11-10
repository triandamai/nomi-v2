import { apiFetch, type ApiEvent } from './api';

/**
 * A file the API keeps (a profile picture, a crew recording), read through it: it lives in the
 * bucket or on the server's disk, and the browser never needs the bucket to be public. Someone
 * signed out can still see what has a public copy (`publicPath`).
 */
export async function proxyFile(event: ApiEvent, path: string, options: { publicPath?: string; fallbackType: string }): Promise<Response> {
	let response = await apiFetch(event, path, undefined, { signedOut: 'return' });
	if (response.status === 401 && options.publicPath) response = await apiFetch(event, options.publicPath, undefined, { signedOut: 'return' });
	if (!response.ok) return new Response(null, { status: response.status === 401 ? 401 : 404 });
	return new Response(response.body, {
		headers: {
			'content-type': response.headers.get('content-type') ?? options.fallbackType,
			'cache-control': response.headers.get('cache-control') ?? 'private, max-age=3600',
			'x-content-type-options': 'nosniff',
		},
	});
}
