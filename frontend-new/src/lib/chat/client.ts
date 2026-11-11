// The chat page's calls to its own endpoints (src/lib/server/chatRoutes.ts), from the browser:
// JSON in and out, and every way one can go wrong told apart. Offline (it never reached the
// server) queues a message; signed out (401) keeps the draft and asks to sign in again; anything
// else carries the server's own words.

export type CallResult<T> = { ok: true; data: T } | { ok: false; status: number; message: string; signedOut: boolean; offline: boolean };

type Init = { method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'; body?: unknown };

export async function callJson<T>(path: string, init: Init = {}, fetchFn: typeof fetch = fetch): Promise<CallResult<T>> {
	let response: Response;
	try {
		response = await fetchFn(path, {
			method: init.method ?? 'GET',
			headers: { 'content-type': 'application/json' },
			...(init.body !== undefined && { body: JSON.stringify(init.body) }),
		});
	} catch {
		return { ok: false, status: 0, message: '', signedOut: false, offline: true };
	}
	const body = (await response.json().catch(() => null)) as unknown;
	if (response.ok) return { ok: true, data: body as T };
	const error = body && typeof body === 'object' && typeof (body as { error?: unknown }).error === 'string' ? (body as { error: string }).error : '';
	return { ok: false, status: response.status, message: error, signedOut: response.status === 401, offline: false };
}
