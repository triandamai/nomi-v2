import { isRedirect, json } from '@sveltejs/kit';
import type { ApiEvent } from './api';
import * as chat from './chat';

// The chat page's own JSON endpoints (/chat/[id]/api/…, and POST /chat to start one): the page
// calls them with fetch, they call the backend as the person through `chat`. A sign-in that can't
// be renewed answers 401 `signed_out` rather than redirecting the fetch, so the page can keep the
// draft and say so (Session Expired); anything the backend refused comes back with its words.

type Body = Record<string, unknown>;

const bad = (error = 'bad_request') => json({ error }, { status: 400 });

async function read(request: Request): Promise<Body> {
	const parsed = await request.json().catch(() => null);
	return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? (parsed as Body) : {};
}

function answer<T>(outcome: chat.Outcome<T>): Response {
	if (outcome.ok) return json(outcome.data ?? null);
	return json({ error: outcome.message }, { status: outcome.status });
}

/** A sign-in that ran out is a 401 here; everything else is the handler's own answer. */
async function signedIn(run: () => Promise<Response>): Promise<Response> {
	try {
		return await run();
	} catch (error) {
		if (isRedirect(error)) return json({ error: 'signed_out' }, { status: 401 });
		throw error;
	}
}

const text = (value: unknown): value is string => typeof value === 'string';

function sessionChange(body: Body): chat.SessionChange | null {
	if (text(body.title)) return { title: body.title };
	if (typeof body.pinned === 'boolean') return { pinned: body.pinned };
	if ('in_project_id' in body && (body.in_project_id === null || text(body.in_project_id))) return { in_project_id: body.in_project_id };
	return null;
}

export function chatRequest(event: ApiEvent, sessionId: string, path: string[], request: Request): Promise<Response> {
	return signedIn(async () => {
		const method = request.method;
		const [first, messageId, action] = path;
		const route = `${method} ${first ?? ''}${messageId ? '/:id' : ''}${action ? `/${action}` : ''}`;
		switch (route) {
			case 'GET messages': {
				const page = await chat.messagePage(event, sessionId, new URL(request.url).searchParams.get('before'));
				return page.messages ? json({ messages: page.messages, hasOlder: page.hasOlder }) : json({ error: 'not_found' }, { status: page.status });
			}
			case 'POST messages': {
				const body = await read(request);
				return text(body.text) ? answer(await chat.sendMessage(event, sessionId, body.text)) : bad();
			}
			case 'GET messages/:id':
				return answer(await chat.message(event, sessionId, messageId));
			case 'PATCH messages/:id': {
				const body = await read(request);
				return text(body.text) ? answer(await chat.editMessage(event, sessionId, messageId, body.text)) : bad();
			}
			case 'POST messages/:id/regenerate': {
				const { style = 'again' } = await read(request);
				return chat.isStyle(style) ? answer(await chat.regenerate(event, sessionId, messageId, style)) : bad();
			}
			case 'POST messages/:id/retry':
				return answer(await chat.retryRun(event, sessionId, messageId));
			case 'GET messages/:id/versions':
				return answer(await chat.messageVersions(event, sessionId, messageId));
			case 'PUT messages/:id/feedback': {
				const body = await read(request);
				if (body.rating !== 'up' && body.rating !== 'down') return bad();
				const reasons = Array.isArray(body.reasons) ? body.reasons.filter(text) : [];
				return answer(await chat.setFeedback(event, sessionId, messageId, { rating: body.rating, reasons, note: text(body.note) ? body.note : '', includeChat: body.includeChat === true }));
			}
			case 'DELETE messages/:id/feedback':
				return answer(await chat.setFeedback(event, sessionId, messageId, { rating: null }));
			case 'PATCH session': {
				const change = sessionChange(await read(request));
				return change ? answer(await chat.updateSession(event, sessionId, change)) : bad();
			}
			case 'DELETE session':
				return answer(await chat.deleteSession(event, sessionId));
			case 'PUT thinking': {
				const { level } = await read(request);
				return chat.isThinkingLevel(level) ? answer(await chat.setThinkingLevel(event, sessionId, level)) : bad();
			}
			case 'PUT model': {
				const { key } = await read(request);
				return text(key) && /^(admin|conn):/.test(key) ? answer(await chat.setSessionModel(event, sessionId, key)) : bad();
			}
			case 'POST share':
				return answer(await chat.shareChat(event, sessionId));
			case 'DELETE share/:id':
				return answer(await chat.revokeShare(event, sessionId, messageId));
			default:
				return json({ error: 'not_found' }, { status: 404 });
		}
	});
}

/** POST /chat: a new chat with its first message (Home, and /chat/new). */
export function startRequest(event: ApiEvent, request: Request): Promise<Response> {
	return signedIn(async () => {
		const body = await read(request);
		if (!text(body.text) || !body.text.trim()) return bad();
		return answer(await chat.startChat(event, { text: body.text, model: text(body.model) ? body.model : undefined, thinking: text(body.thinking) ? body.thinking : undefined }));
	});
}
