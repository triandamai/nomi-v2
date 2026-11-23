import { json } from '@sveltejs/kit';
import { loadBoard, moveIssue, retryIssue } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

// The workspace's Issues sheet, from the browser: the board (GET), and moving an issue or handing
// a failed one back to the crew (POST `{ action, number, column?, index? }`).

export const GET: RequestHandler = async (event) => {
	const board = await loadBoard(event, event.params.id);
	return board ? json(board) : new Response(null, { status: 503 });
};

export const POST: RequestHandler = async (event) => {
	const body = (await event.request.json().catch(() => null)) as { action?: string; number?: unknown; column?: unknown; index?: unknown } | null;
	const number = Number(body?.number);
	if (!body || !Number.isSafeInteger(number) || number <= 0) return new Response(null, { status: 400 });
	let sent;
	if (body.action === 'move' && typeof body.column === 'string' && Number.isSafeInteger(body.index)) sent = await moveIssue(event, event.params.id, number, body.column, body.index as number);
	else if (body.action === 'retry') sent = await retryIssue(event, event.params.id, number);
	else return new Response(null, { status: 400 });
	return sent.ok ? new Response(null, { status: 204 }) : new Response(sent.message, { status: sent.status });
};
