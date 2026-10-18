// Calls one of the current page's provider form actions from script (connect, disconnect,
// list a connection's models…), without touching the page's `form`, which belongs to its own
// forms (on the chat page, the message box's error).
import { deserialize } from '$app/forms';
import { goto, invalidateAll } from '$app/navigation';

/** Reloads the page's data after a change (kept here so components have one seam to mock). */
export const reloadData = () => invalidateAll();

export type ActionOutcome<T> = { ok: true; data: T } | { ok: false; error: string; models: string[] };

export async function postAction<T>(name: string, body: FormData): Promise<ActionOutcome<T>> {
	try {
		const response = await fetch(`?/${name}`, { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' } });
		const result = deserialize(await response.text());
		if (result.type === 'success') return { ok: true, data: (result.data ?? {}) as T };
		if (result.type === 'redirect') {
			// Signed out meanwhile: off to log in.
			await goto(result.location);
		} else if (result.type === 'failure') {
			const error = typeof result.data?.error === 'string' ? result.data.error : 'failed';
			const models = Array.isArray(result.data?.models) ? (result.data.models as string[]) : [];
			return { ok: false, error, models };
		}
	} catch {
		// Offline, or the server answered with something that isn't an action result.
	}
	return { ok: false, error: 'failed', models: [] };
}
