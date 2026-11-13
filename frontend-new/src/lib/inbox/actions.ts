import { deserialize } from '$app/forms';
import { goto } from '$app/navigation';
import { m } from '$lib/paraglide/messages';

// The Inbox page's actions, called from its rows and menus and from the top bar's Mark all read
// (which the layout draws). A sign-in that has run out comes back as the action's redirect.

/** The quick reminder's field, for the top bar's "New reminder" to put focus in. */
export const QUICK_REMINDER = 'quick-reminder';

export type InboxActionResult = { ok: true; data: Record<string, unknown> } | { ok: false; message: string; field?: string };

export async function inboxAction(name: string, fields: Record<string, string> = {}, options: { keepalive?: boolean } = {}): Promise<InboxActionResult> {
	const body = new FormData();
	for (const [key, value] of Object.entries(fields)) body.set(key, value);
	const response = await fetch(`/inbox?/${name}`, { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' }, keepalive: options.keepalive }).catch(() => null);
	const result = response ? await response.text().then(deserialize).catch(() => null) : null;
	if (result?.type === 'success') return { ok: true, data: result.data ?? {} };
	if (result?.type === 'redirect') {
		await goto(result.location);
		return { ok: false, message: '' };
	}
	if (result?.type === 'failure') {
		const said = (result.data ?? {}) as { message?: string; field?: string };
		return { ok: false, message: said.message || m.inbox_action_failed(), field: said.field };
	}
	return { ok: false, message: m.inbox_action_failed() };
}
