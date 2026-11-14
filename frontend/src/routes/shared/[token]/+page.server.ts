import { fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { apiFetch, apiUrl } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import { isReportReason } from '$lib/sharedPage';
import type { Actions, PageServerLoad } from './$types';

type SharedChat = {
	title: string | null;
	scope: 'reply' | 'chat';
	masked: boolean;
	expires_at: string | null;
	messages: { sender: 'user' | 'assistant'; agent_display_name: string | null; content: string; created_at: string }[];
	/** Who shared it: their name, or its initials when the link is masked. */
	shared_by: string;
	shared_at: string;
	/** Only when the reader made this link: their banner's facts. */
	own?: boolean;
	share_id?: string;
	session_id?: string;
	views?: number;
};

/** The longest note a report sends (the backend keeps the first 1000 characters). */
const REPORT_NOTE_MAX = 1000;

const bearer = (accessToken: string | undefined): Record<string, string> => (accessToken ? { Authorization: `Bearer ${accessToken}` } : {});

// A shared reply or chat, read-only. Public when its owner said anyone may view; otherwise the
// reader's sign-in goes along, and without one the page asks for it.
export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await fetch(apiUrl(`/api/shared/${encodeURIComponent(params.token)}`), {
		headers: bearer(cookies.get('access_token')),
	}).catch(() => null);
	if (response?.status === 401) return { state: 'sign_in' as const };
	if (!response?.ok) return { state: 'gone' as const };
	const chat = (await response.json()) as SharedChat;
	const messages = await Promise.all(chat.messages.map(async (message) => ({ ...message, html: await renderMarkdown(message.content) })));
	return { state: 'ready' as const, chat: { ...chat, messages } };
};

export const actions: Actions = {
	/** The owner stops sharing from their own preview: the link is revoked. */
	stop: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		const sessionId = String(form.get('session_id') ?? '');
		const shareId = String(form.get('share_id') ?? '');
		if (!sessionId || !shareId) return fail(400, { stopError: m.shared_stop_failed() });
		const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(sessionId)}/shares/${encodeURIComponent(shareId)}`, {
			method: 'DELETE',
		});
		if (!response.ok) return fail(response.status, { stopError: m.shared_stop_failed() });
		return { stopped: true };
	},
	/** Anyone viewing flags the page; open without a sign-in, a few times an hour. */
	report: async ({ request, params, cookies, fetch, getClientAddress }) => {
		const form = await request.formData();
		const reason = form.get('reason');
		const note = String(form.get('note') ?? '')
			.trim()
			.slice(0, REPORT_NOTE_MAX);
		if (!isReportReason(reason)) return fail(400, { reportError: m.shared_report_pick() });
		const response = await fetch(apiUrl(`/api/shared/${encodeURIComponent(params.token)}/report`), {
			method: 'POST',
			headers: { 'content-type': 'application/json', 'x-forwarded-for': getClientAddress(), ...bearer(cookies.get('access_token')) },
			body: JSON.stringify({ reason, note: note || null }),
		}).catch(() => null);
		if (response?.status === 429) return fail(429, { reportError: m.shared_report_too_many() });
		if (!response?.ok) return fail(response?.status ?? 502, { reportError: m.shared_report_failed() });
		return { reported: true };
	},
};
