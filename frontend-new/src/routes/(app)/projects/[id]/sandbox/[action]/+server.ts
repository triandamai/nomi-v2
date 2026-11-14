import { json } from '@sveltejs/kit';
import { loadSandbox, sandboxBeat, sandboxLogs, startSandbox, stopSandbox } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

// The workspace's sandbox, from the browser: its status, its logs from an offset, the heartbeat
// that keeps it awake (GET), and starting or stopping it (POST). Nothing else reaches the backend.

export const GET: RequestHandler = async (event) => {
	const { id, action } = event.params;
	if (action === 'status') {
		const status = await loadSandbox(event, id);
		return status ? json(status) : new Response(null, { status: 503 });
	}
	if (action === 'logs') {
		const offset = Number.parseInt(event.url.searchParams.get('offset') ?? '0', 10);
		const logs = await sandboxLogs(event, id, Number.isSafeInteger(offset) && offset > 0 ? offset : 0);
		return logs ? json(logs) : new Response(null, { status: 503 });
	}
	if (action === 'beat') return new Response(null, { status: (await sandboxBeat(event, id)) ? 204 : 503 });
	return new Response(null, { status: 404 });
};

export const POST: RequestHandler = async (event) => {
	const { id, action } = event.params;
	if (action !== 'start' && action !== 'stop') return new Response(null, { status: 404 });
	const sent = await (action === 'start' ? startSandbox : stopSandbox)(event, id);
	return sent.ok ? new Response(null, { status: 204 }) : new Response(sent.message, { status: sent.status });
};
