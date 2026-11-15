import { error, json } from '@sveltejs/kit';
import type { Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { readOrderRequest } from './orderRequest';

/**
 * Sends a quote or checkout request to the backend and hands its answer back as it is: the JSON on
 * success, the backend's plain-text message and status otherwise (a bad code, or 503 when
 * payments aren't set up).
 */
export async function forwardOrder(fetchFn: typeof fetch, cookies: Cookies, request: Request, path: string): Promise<Response> {
	const order = readOrderRequest(await request.json().catch(() => null));
	if (!order) throw error(400, 'invalid order');
	const response = await apiFetch(fetchFn, cookies, path, { method: 'POST', body: JSON.stringify(order) });
	if (response.ok) return json(await response.json());
	return new Response(await response.text(), { status: response.status, headers: { 'content-type': 'text/plain; charset=utf-8' } });
}
