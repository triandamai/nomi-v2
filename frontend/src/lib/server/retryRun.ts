import { fail, type Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';

/** Tries a failed run again from its error bubble (`messageId`), queuing what it names. */
export async function retryRun(fetch: typeof globalThis.fetch, cookies: Cookies, sessionId: string, data: FormData) {
	const messageId = data.get('messageId');
	if (typeof messageId !== 'string' || !messageId) return fail(400, { error: m.run_failed_retry_error() });
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/messages/${messageId}/retry`, { method: 'POST' });
	if (!response.ok) {
		// 409: the crew is already on it again, or there's nothing left to rerun.
		const message = response.status === 409 ? await response.text() : '';
		return fail(response.status, { error: message || m.run_failed_retry_error() });
	}
	return { retried: true };
}
