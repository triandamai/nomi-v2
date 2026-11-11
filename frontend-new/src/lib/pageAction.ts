import { deserialize } from '$app/forms';
import { goto } from '$app/navigation';
import { m } from '$lib/paraglide/messages';

// Calls a page's form action from script (a row's menu, a switch, the top bar), with the same
// result a form would get. A sign-in that has run out comes back as the action's redirect, which
// is followed; a failure says what the action said, or that it didn't go through.

export type PageActionResult = { ok: true; data: Record<string, unknown> } | { ok: false; message: string; field?: string };

export async function pageAction(path: string, name: string, fields: Record<string, string> = {}, options: { keepalive?: boolean } = {}): Promise<PageActionResult> {
	const body = new FormData();
	for (const [key, value] of Object.entries(fields)) body.set(key, value);
	const response = await fetch(`${path}?/${name}`, { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' }, keepalive: options.keepalive }).catch(() => null);
	const result = response ? await response.text().then(deserialize).catch(() => null) : null;
	if (result?.type === 'success') return { ok: true, data: result.data ?? {} };
	if (result?.type === 'redirect') {
		await goto(result.location);
		return { ok: false, message: '' };
	}
	if (result?.type === 'failure') {
		const said = (result.data ?? {}) as { message?: string; field?: string };
		return { ok: false, message: said.message || m.action_failed(), field: said.field };
	}
	return { ok: false, message: m.action_failed() };
}
