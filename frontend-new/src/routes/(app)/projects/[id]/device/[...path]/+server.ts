import { apiFetch } from '$lib/server/api';
import { devicePath } from '$lib/projects/device';
import type { RequestHandler } from './$types';

// The device runtime (lib/projects/device.svelte.ts) reaches the backend through here: the
// project's files, its check-in with Koda's commands, their results and the dev backend's address.
// Only those paths pass (`DEVICE_PATHS`); anything else is a 404 without a call.
const proxy: RequestHandler = async (event) => {
	const path = devicePath(event.params.path);
	if (!path) return new Response(null, { status: 404 });
	const init: RequestInit = { method: event.request.method, signal: event.request.signal };
	if (event.request.method === 'POST') {
		init.body = await event.request.text();
		init.headers = { 'content-type': 'application/json' };
	}
	const response = await apiFetch(event, `/api/projects/${encodeURIComponent(event.params.id)}/${path}${event.url.search}`, init);
	return new Response(response.body, { status: response.status, headers: { 'content-type': response.headers.get('content-type') ?? 'application/json', 'cache-control': 'no-store' } });
};

export const GET = proxy;
export const POST = proxy;
