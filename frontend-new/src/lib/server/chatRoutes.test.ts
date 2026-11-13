import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { chatRequest, startRequest } = await import('./chatRoutes');

function eventWith(answer: Parameters<typeof fakeBackend>[0], cookies: Record<string, string> = {}) {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' }, cookies }) };
}

const request = (method: string, body?: unknown, url = 'http://localhost/chat/s1/api/x') =>
	new Request(url, { method, ...(body !== undefined && { body: JSON.stringify(body), headers: { 'content-type': 'application/json' } }) });

describe('the chat page’s own endpoints', () => {
	it('makes a share link with its settings, changes them, and turns it off', async () => {
		const { backend, event } = eventWith(() => json({ id: 'sh1', token: 't1' }));
		const settings = { scope: 'reply', message_id: 'm1', mask: true, anyone: false, expires_in_days: 30 };
		await chatRequest(event, 's1', ['share'], request('POST', settings));
		await chatRequest(event, 's1', ['share', 'sh1'], request('PUT', { ...settings, expires_in_days: null }));
		await chatRequest(event, 's1', ['share', 'sh1'], request('DELETE'));
		expect(backend.calls.map((call) => [call.method, call.url, call.body])).toEqual([
			['POST', 'http://api.test/api/sessions/s1/shares', settings],
			['PUT', 'http://api.test/api/sessions/s1/shares/sh1', { ...settings, expires_in_days: null }],
			['DELETE', 'http://api.test/api/sessions/s1/shares/sh1', undefined],
		]);
		expect((await chatRequest(event, 's1', ['share'], request('POST', { scope: 'all' }))).status).toBe(400);
		expect((await chatRequest(event, 's1', ['share'], request('POST', { scope: 'chat', expires_in_days: 3 }))).status).toBe(400);
	});

	it('records a decision on an approval, remembered or not', async () => {
		const { backend, event } = eventWith(() => json({}));
		await chatRequest(event, 's1', ['messages', 'm1', 'approval'], request('PUT', { decision: 'approve', remember: true }));
		expect(backend.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/sessions/s1/messages/m1/approval', body: { decision: 'approve', remember: true } });
		expect((await chatRequest(event, 's1', ['messages', 'm1', 'approval'], request('PUT', { decision: 'maybe' }))).status).toBe(400);
	});

	it('reads a crew member’s plans, and ticks a step off', async () => {
		const { backend, event } = eventWith(() => json({ plans: [], build: null }));
		const listed = await chatRequest(event, 's1', ['agent-plans', 'a1'], request('GET'));
		expect(await listed.json()).toEqual({ plans: [], build: null });
		await chatRequest(event, 's1', ['agent-plans', 'a1', 'p1', 'items', '2'], request('PUT', { done: true }));
		expect(backend.calls.map((call) => [call.method, call.url, call.body])).toEqual([
			['GET', 'http://api.test/api/sessions/s1/agent-plans/a1', undefined],
			['PUT', 'http://api.test/api/sessions/s1/agent-plans/a1/p1/items/2', { done: true }],
		]);
		expect((await chatRequest(event, 's1', ['agent-plans', 'a1', 'p1', 'items', 'x'], request('PUT', { done: true }))).status).toBe(400);
	});

	it('marks a reminder done or snoozes it', async () => {
		const { backend, event } = eventWith(() => json({}));
		await chatRequest(event, 's1', ['reminders', 'r1'], request('POST', { action: 'snooze', minutes: 30 }));
		expect(backend.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/reminders/r1', body: { action: 'snooze', minutes: 30 } });
		expect((await chatRequest(event, 's1', ['reminders', 'r1'], request('POST', { action: 'later' }))).status).toBe(400);
	});

	it('sends, and answers what was sent', async () => {
		const { backend, event } = eventWith(() => json({ user_message: { id: 'u' } }, 202));
		const response = await chatRequest(event, 's1', ['messages'], request('POST', { text: 'hi' }));
		expect(response.status).toBe(200);
		expect(await response.json()).toMatchObject({ message: { id: 'u' }, stopped: false });
		expect(backend.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/sessions/s1/messages', body: { text: 'hi' } });
	});

	it('loads an earlier page before a message', async () => {
		const { backend, event } = eventWith(() => json({ messages: [], has_more: false }));
		const response = await chatRequest(event, 's1', ['messages'], request('GET', undefined, 'http://localhost/chat/s1/api/messages?before=m5'));
		expect(await response.json()).toEqual({ messages: [], hasOlder: false });
		expect(backend.calls[0].url).toContain('before=m5');
	});

	it('routes each message action to its backend call', async () => {
		const { backend, event } = eventWith(() => json({ message_id: 'u', version: 2, total: 2, edited_at: null }));
		await chatRequest(event, 's1', ['messages', 'm1', 'regenerate'], request('POST', { style: 'longer' }));
		await chatRequest(event, 's1', ['messages', 'm1'], request('PATCH', { text: 'edited' }));
		await chatRequest(event, 's1', ['messages', 'm1', 'retry'], request('POST'));
		await chatRequest(event, 's1', ['messages', 'm1', 'feedback'], request('PUT', { rating: 'up' }));
		await chatRequest(event, 's1', ['session'], request('PATCH', { pinned: true }));
		await chatRequest(event, 's1', ['thinking'], request('PUT', { level: 'high' }));
		await chatRequest(event, 's1', ['model'], request('PUT', { key: 'admin:1' }));
		expect(backend.calls.map((call) => `${call.method} ${call.url.replace('http://api.test/api/sessions/s1', '')}`)).toEqual([
			'POST /messages/m1/regenerate',
			'PATCH /messages/m1',
			'POST /messages/m1/retry',
			'PUT /messages/m1/feedback',
			'PATCH ',
			'PUT /thinking',
			'PUT /model',
		]);
	});

	it('refuses what it doesn’t know, or a body that isn’t right', async () => {
		const { backend, event } = eventWith(() => json({}));
		expect((await chatRequest(event, 's1', ['nope'], request('GET'))).status).toBe(404);
		expect((await chatRequest(event, 's1', ['messages', 'm1', 'regenerate'], request('POST', { style: 'louder' }))).status).toBe(400);
		expect((await chatRequest(event, 's1', ['thinking'], request('PUT', { level: 'off' }))).status).toBe(400);
		expect((await chatRequest(event, 's1', ['session'], request('PATCH', { colour: 'red' }))).status).toBe(400);
		expect(backend.calls).toHaveLength(0);
	});

	it('passes the backend’s refusal on, with its words', async () => {
		const { event } = eventWith(() => new Response('the crew is already working in this chat', { status: 409 }));
		const response = await chatRequest(event, 's1', ['messages', 'm1', 'regenerate'], request('POST', { style: 'again' }));
		expect(response.status).toBe(409);
		expect(await response.json()).toEqual({ error: 'the crew is already working in this chat' });
	});

	it('says signed_out (401) when the sign-in can’t be renewed, instead of redirecting a fetch', async () => {
		const { event } = eventWith(() => new Response('', { status: 401 }));
		const response = await chatRequest(event, 's1', ['messages'], request('POST', { text: 'hi' }));
		expect(response.status).toBe(401);
		expect(await response.json()).toEqual({ error: 'signed_out' });
	});
});

describe('starting a chat', () => {
	it('creates it with its first message and says where it is', async () => {
		const { event } = eventWith((call) => (call.url.endsWith('/api/sessions') ? json({ session_id: 'new' }, 201) : json({ user_message: {} }, 202)));
		const response = await startRequest(event, request('POST', { text: 'hi' }, 'http://localhost/chat'));
		expect(await response.json()).toEqual({ sessionId: 'new', sent: true });
	});

	it('refuses an empty message', async () => {
		const { event } = eventWith(() => json({}));
		expect((await startRequest(event, request('POST', { text: ' ' }, 'http://localhost/chat'))).status).toBe(400);
	});
});
