import { isRedirect } from '@sveltejs/kit';
import type {
	HomeSummary,
	MessageItem,
	MessageVersion,
	ModelChoice,
	ModelOptions,
	OrgList,
	ProjectList,
	RenderedMessage,
	RerunResult,
	SendResult,
	SessionSummary,
	ThinkingLevel,
} from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';
import { readApiError } from './apiError';
import { renderMarkdown } from './markdown';

// The chat's calls to the backend, as the person: a page of messages (rendered), one message,
// sending, the chat's own settings (name, pin, project, model, thinking), feedback, running the
// latest turn again, retrying a failed run, and the projects a chat can move into. Each answers
// `{ ok, data }` or `{ ok: false, status, message }` with what the backend said; a sign-in that
// can't be renewed goes through as apiFetch's redirect. Logic and API use from the old app
// (frontend/src/lib/server/{fetchMessagePage,renameSession,updateSession,retryRun,thinking,modelOptions}.ts).

export type Outcome<T> = { ok: true; data: T } | { ok: false; status: number; message: string };

/** How many messages a chat opens with, and loads per earlier page. */
export const MESSAGE_PAGE_SIZE = 50;

const THINKING_LEVELS: readonly string[] = ['low', 'medium', 'high', 'max'] satisfies ThinkingLevel[];
const STYLES = ['again', 'shorter', 'longer', 'list'] as const;
export type RegenerateStyle = (typeof STYLES)[number];
export const isStyle = (value: unknown): value is RegenerateStyle => STYLES.includes(value as RegenerateStyle);
export const isThinkingLevel = (value: unknown): value is ThinkingLevel => THINKING_LEVELS.includes(value as string);

/** A backend that can't be reached answers like one that's down. */
async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

/** What the backend said, a plain sentence or problem details alike. */
async function failed(response: Response): Promise<{ ok: false; status: number; message: string }> {
	return { ok: false, status: response.status, message: (await readApiError(response)).message };
}

async function outcome<T>(response: Response, read: (response: Response) => Promise<T>): Promise<Outcome<T>> {
	if (!response.ok) return failed(response);
	return { ok: true, data: await read(response) };
}

const body = (value: unknown): RequestInit['body'] => JSON.stringify(value);
const nothing = async () => undefined;

export async function render(message: MessageItem): Promise<RenderedMessage> {
	return { ...message, content_html: await renderMarkdown(message.content) };
}

/**
 * A page of a chat's messages, oldest first and rendered, ending just before `before` (or at the
 * newest). The backend never starts a page inside a thread. `messages` is null when the chat
 * can't be read (404: gone, or never theirs).
 */
export async function messagePage(event: ApiEvent, sessionId: string, before: string | null = null) {
	const query = new URLSearchParams({ limit: String(MESSAGE_PAGE_SIZE) });
	if (before) query.set('before', before);
	const response = await call(event, `/api/sessions/${sessionId}/messages?${query}`);
	if (!response.ok) return { status: response.status, messages: null, hasOlder: false } as const;
	const page = (await response.json()) as { messages: MessageItem[]; has_more?: boolean };
	return { status: response.status, messages: await Promise.all(page.messages.map(render)), hasOlder: Boolean(page.has_more) };
}

/**
 * One message, rendered (a live update fetches what changed). The backend names a teammate's
 * message in a shared project chat only in the list, so a person's message is looked up there too
 * (BACKEND_TODO.md): without it, a teammate's new message would look like the person's own.
 */
export async function message(event: ApiEvent, sessionId: string, messageId: string): Promise<Outcome<RenderedMessage>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/messages/${messageId}`), async (response) => {
		const found = (await response.json()) as MessageItem;
		if (found.sender === 'user' && !found.author_name) {
			const listed = await call(event, `/api/sessions/${sessionId}/messages?limit=20`);
			const page = listed.ok ? ((await listed.json()) as { messages: MessageItem[] }) : null;
			const author = page?.messages.find((one) => one.id === found.id)?.author_name;
			if (author) found.author_name = author;
		}
		return render(found);
	});
}

/** The person's chats, latest first; [] when they can't be read. */
/** What Home's Today strip sums up; null when it can't be read (the strip isn't drawn). */
export async function homeSummary(event: ApiEvent): Promise<HomeSummary | null> {
	const response = await call(event, '/api/home');
	if (!response.ok) return null;
	return (await response.json().catch(() => null)) as HomeSummary | null;
}

export async function listSessions(event: ApiEvent): Promise<SessionSummary[]> {
	const response = await call(event, '/api/sessions');
	if (!response.ok) return [];
	return ((await response.json()) as { sessions: SessionSummary[] }).sessions;
}

/** The Chats page's list; null when the backend couldn't give it (not the same as no chats). */
export async function chatList(event: ApiEvent): Promise<SessionSummary[] | null> {
	const response = await call(event, '/api/sessions');
	if (!response.ok) return null;
	return ((await response.json().catch(() => null)) as { sessions?: SessionSummary[] } | null)?.sessions ?? null;
}

/** A chat's name, pin and project: there's no single-chat endpoint, so from the list. */
export async function findSession(event: ApiEvent, sessionId: string): Promise<SessionSummary | null> {
	return (await listSessions(event)).find((session) => session.id === sessionId) ?? null;
}

export async function sendMessage(event: ApiEvent, sessionId: string, text: string): Promise<Outcome<{ message: MessageItem; reply: MessageItem | null; stopped: boolean }>> {
	if (!text.trim()) return { ok: false, status: 400, message: 'text must not be empty' };
	const response = await call(event, `/api/sessions/${sessionId}/messages`, { method: 'POST', body: body({ text }) });
	return outcome(response, async (sent) => {
		const result = (await sent.json()) as SendResult;
		// A stop is answered on the spot by the supervisor: no turn is coming.
		return { message: result.user_message, reply: result.supervisor_reply ?? null, stopped: Boolean(result.supervisor_reply) };
	});
}

/**
 * A new chat with its first message: created, set to the model and thinking level picked for it
 * (either failing leaves the person's default), then the message. A chat whose message didn't go
 * still opens, so it can be sent again from there.
 */
export async function startChat(event: ApiEvent, start: { text: string; model?: string; thinking?: string }): Promise<Outcome<{ sessionId: string; sent: boolean }>> {
	if (!start.text.trim()) return { ok: false, status: 400, message: 'text must not be empty' };
	const created = await call(event, '/api/sessions', { method: 'POST' });
	if (!created.ok) return failed(created);
	const { session_id: sessionId } = (await created.json()) as { session_id: string };
	if (start.model) await call(event, `/api/sessions/${sessionId}/model`, { method: 'PUT', body: body({ key: start.model }) });
	if (start.thinking && isThinkingLevel(start.thinking)) await call(event, `/api/sessions/${sessionId}/thinking`, { method: 'PUT', body: body({ level: start.thinking }) });
	const sent = await call(event, `/api/sessions/${sessionId}/messages`, { method: 'POST', body: body({ text: start.text }) });
	return { ok: true, data: { sessionId, sent: sent.ok } };
}

export type SessionChange = { title: string } | { pinned: boolean } | { in_project_id: string | null };

/** Renames, pins or unpins, or moves a chat into a project (or out with null). */
export async function updateSession(event: ApiEvent, sessionId: string, change: SessionChange): Promise<Outcome<{ title: string | null; pinned: boolean; in_project_id: string | null }>> {
	const sent = 'title' in change ? { title: change.title.trim() } : change;
	if ('title' in sent && !sent.title) return { ok: false, status: 400, message: 'title must not be empty' };
	return outcome(await call(event, `/api/sessions/${sessionId}`, { method: 'PATCH', body: body(sent) }), (response) => response.json());
}

export async function deleteSession(event: ApiEvent, sessionId: string): Promise<Outcome<void>> {
	return outcome(await call(event, `/api/sessions/${sessionId}`, { method: 'DELETE' }), nothing);
}

export type Feedback = { rating: 'up' | 'down' | null; reasons?: string[]; note?: string; includeChat?: boolean };

/** A like or a dislike (with its reasons and note), or neither (taken back). */
export async function setFeedback(event: ApiEvent, sessionId: string, messageId: string, feedback: Feedback): Promise<Outcome<void>> {
	const path = `/api/sessions/${sessionId}/messages/${messageId}/feedback`;
	if (!feedback.rating) return outcome(await call(event, path, { method: 'DELETE' }), nothing);
	const note = feedback.note?.trim();
	return outcome(
		await call(event, path, { method: 'PUT', body: body({ rating: feedback.rating, reasons: feedback.reasons ?? [], note: note || null, include_chat: feedback.includeChat ?? false }) }),
		nothing,
	);
}

/** Asks for the latest turn's reply again, in a style. */
export async function regenerate(event: ApiEvent, sessionId: string, messageId: string, style: RegenerateStyle): Promise<Outcome<RerunResult>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/messages/${messageId}/regenerate`, { method: 'POST', body: body({ style }) }), (response) => response.json());
}

/** Saves the person's latest message with new text, and runs its turn again. */
export async function editMessage(event: ApiEvent, sessionId: string, messageId: string, text: string): Promise<Outcome<RerunResult>> {
	if (!text.trim()) return { ok: false, status: 400, message: 'text must not be empty' };
	return outcome(await call(event, `/api/sessions/${sessionId}/messages/${messageId}`, { method: 'PATCH', body: body({ text }) }), (response) => response.json());
}

/** The earlier takes of the person's message, their replies rendered. */
export async function messageVersions(event: ApiEvent, sessionId: string, messageId: string): Promise<Outcome<MessageVersion[]>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/messages/${messageId}/versions`), async (response) => {
		const { versions } = (await response.json()) as { versions: (Omit<MessageVersion, 'replies'> & { replies: Omit<MessageVersion['replies'][number], 'content_html'>[] })[] };
		return Promise.all(
			versions.map(async (version) => ({ ...version, replies: await Promise.all(version.replies.map(async (reply) => ({ ...reply, content_html: await renderMarkdown(reply.content) }))) })),
		);
	});
}

/** Tries a failed run again from its error bubble, which the backend then removes. */
export async function retryRun(event: ApiEvent, sessionId: string, messageId: string): Promise<Outcome<void>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/messages/${messageId}/retry`, { method: 'POST' }), nothing);
}

/** The chat's thinking level; null (the picker hides) when it can't be read. */
export async function thinkingLevel(event: ApiEvent, sessionId: string): Promise<ThinkingLevel | null> {
	const response = await call(event, `/api/sessions/${sessionId}/thinking`);
	if (!response.ok) return null;
	const { level } = (await response.json()) as { level: string };
	return isThinkingLevel(level) ? level : null;
}

export async function setThinkingLevel(event: ApiEvent, sessionId: string, level: ThinkingLevel): Promise<Outcome<void>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/thinking`, { method: 'PUT', body: body({ level }) }), nothing);
}

/** The models a chat can use, grouped by who pays; nothing when they can't be read. */
export async function modelOptions(event: ApiEvent): Promise<ModelOptions> {
	const response = await call(event, '/api/llm/model-options');
	if (!response.ok) return { groups: [], current: null };
	const options = (await response.json().catch(() => null)) as Partial<ModelOptions> | null;
	return { groups: Array.isArray(options?.groups) ? options.groups : [], current: options?.current ?? null };
}

/** The model this chat answers with; null when it's the person's default or can't be read. */
export async function sessionModel(event: ApiEvent, sessionId: string): Promise<ModelChoice | null> {
	const response = await call(event, `/api/sessions/${sessionId}/model`);
	if (!response.ok) return null;
	return ((await response.json().catch(() => null)) as { model?: ModelChoice | null } | null)?.model ?? null;
}

export async function setSessionModel(event: ApiEvent, sessionId: string, key: string): Promise<Outcome<ModelChoice>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/model`, { method: 'PUT', body: body({ key }) }), async (response) => ((await response.json()) as { model: ModelChoice }).model);
}

export type MoveTarget = { id: string; name: string; workspace: string };

/**
 * The projects a chat can be filed under, named with their workspace. Filing a chat only changes
 * where it sits in the person's own list (the backend keeps it private), so MoveDialog's note
 * that a shared workspace's members will see it is never given.
 */
export async function moveTargets(event: ApiEvent): Promise<MoveTarget[]> {
	const [projects, orgs] = await Promise.all([call(event, '/api/projects'), call(event, '/api/orgs')]);
	if (!projects.ok) return [];
	const list = (await projects.json()) as ProjectList;
	const workspaces = orgs.ok ? ((await orgs.json()) as OrgList).orgs : [];
	const workspace = workspaces.find((org) => org.id === list.org_id)?.name ?? '';
	return list.projects.map(({ id, name }) => ({ id, name, workspace }));
}

/** A link to the whole chat (a seam until the share dialog lands): masked, open to anyone, 7 days. */
/** How a link shares: the whole chat or one reply, masked or not, for anyone or signed-in people, for 1, 7 or 30 days or for good. */
export type ShareSettings = { scope: 'chat' | 'reply'; message_id: string | null; mask: boolean; anyone: boolean; expires_in_days: 1 | 7 | 30 | null };

/** What a share link is and what it opens with (`POST`/`PUT …/shares`). */
export type ShareLink = {
	id: string;
	token: string;
	scope: 'chat' | 'reply';
	message_id: string | null;
	mask: boolean;
	anyone: boolean;
	expires_at: string | null;
	preview: string;
	preview_by: string | null;
	chat_messages: number;
};

/** The settings from a request body, or null when they aren't ones the backend takes. */
export function shareSettings(input: Record<string, unknown>): ShareSettings | null {
	if (input.scope !== 'chat' && input.scope !== 'reply') return null;
	const days = input.expires_in_days ?? null;
	if (days !== null && days !== 1 && days !== 7 && days !== 30) return null;
	return {
		scope: input.scope,
		message_id: typeof input.message_id === 'string' ? input.message_id : null,
		mask: input.mask !== false,
		anyone: input.anyone !== false,
		expires_in_days: days as ShareSettings['expires_in_days'],
	};
}

export async function shareChat(event: ApiEvent, sessionId: string, settings: ShareSettings): Promise<Outcome<ShareLink>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/shares`, { method: 'POST', body: body(settings) }), (response) => response.json());
}

export async function updateShare(event: ApiEvent, sessionId: string, shareId: string, settings: ShareSettings): Promise<Outcome<ShareLink>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/shares/${shareId}`, { method: 'PUT', body: body(settings) }), (response) => response.json());
}

export async function revokeShare(event: ApiEvent, sessionId: string, shareId: string): Promise<Outcome<void>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/shares/${shareId}`, { method: 'DELETE' }), nothing);
}

/** A decision on an approval card: approve (and maybe remember it for next time) or deny. */
export async function resolveApproval(event: ApiEvent, sessionId: string, messageId: string, decision: 'approve' | 'deny', remember: boolean): Promise<Outcome<void>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/messages/${messageId}/approval`, { method: 'PUT', body: body({ decision, remember }) }), nothing);
}

/** A crew member's plan in this chat, each version newest last, and its build when it has one. */
export async function agentPlans(event: ApiEvent, sessionId: string, agentSessionId: string): Promise<Outcome<unknown>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/agent-plans/${agentSessionId}`), (response) => response.json());
}

/** A step of a plan ticked off, or back on. */
export async function setPlanItem(event: ApiEvent, sessionId: string, agentSessionId: string, planId: string, index: number, done: boolean): Promise<Outcome<void>> {
	return outcome(await call(event, `/api/sessions/${sessionId}/agent-plans/${agentSessionId}/${planId}/items/${index}`, { method: 'PUT', body: body({ done }) }), nothing);
}

/** A reminder that went off: done, or snoozed for some minutes. */
export async function reminderAction(event: ApiEvent, reminderId: string, action: 'done' | 'snooze', minutes?: number): Promise<Outcome<void>> {
	return outcome(await call(event, `/api/reminders/${reminderId}`, { method: 'POST', body: body({ action, ...(minutes && { minutes }) }) }), nothing);
}

