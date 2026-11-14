import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const chat = await import('./chat');

const message = (id: string, extra: Record<string, unknown> = {}) => ({
	id,
	sender: 'assistant',
	content: `**${id}**`,
	content_blocks: null,
	created_at: '2026-10-21T07:00:00Z',
	my_feedback: null,
	agent_display_name: 'Nomi',
	memory_count: 0,
	...extra,
});

function eventWith(answer: Parameters<typeof fakeBackend>[0]) {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
}

describe('the chat on the server', () => {
	it('loads a page of messages, newest first page, with their markdown rendered', async () => {
		const { backend, event } = eventWith(() => json({ messages: [message('a')], has_more: true }));
		const page = await chat.messagePage(event, 's1', 'm9');
		expect(backend.calls[0].url).toBe('http://api.test/api/sessions/s1/messages?limit=50&before=m9');
		expect(page).toMatchObject({ status: 200, hasOlder: true });
		expect(page.messages?.[0].content_html).toContain('<strong>a</strong>');
	});

	it('says a chat that is gone is gone', async () => {
		const { event } = eventWith(() => new Response('session not found', { status: 404 }));
		expect(await chat.messagePage(event, 's1')).toEqual({ status: 404, messages: null, hasOlder: false });
	});

	it('finds the chat in the list for its title and pin (there is no single-chat endpoint)', async () => {
		const { event } = eventWith(() => json({ sessions: [{ id: 's1', title: 'Rent', pinned: true, in_project_id: null, project_id: null, last_message: null, updated_at: '' }] }));
		expect(await chat.findSession(event, 's1')).toMatchObject({ title: 'Rent', pinned: true });
		expect(await chat.findSession(event, 'nope')).toBeNull();
	});

	it('sends a message, and tells a stop from a turn', async () => {
		const { backend, event } = eventWith(() => json({ user_message: message('u', { sender: 'user' }), supervisor_reply: message('s') }));
		const sent = await chat.sendMessage(event, 's1', 'stop');
		expect(backend.calls[0]).toMatchObject({ url: 'http://api.test/api/sessions/s1/messages', method: 'POST', body: { text: 'stop' } });
		expect(sent).toMatchObject({ ok: true, data: { stopped: true } });
	});

	it('hands back what the backend said when a call fails', async () => {
		const { event } = eventWith(() => new Response('message is too long', { status: 413 }));
		expect(await chat.sendMessage(event, 's1', 'x')).toEqual({ ok: false, status: 413, message: 'message is too long' });
		const { event: down } = eventWith(() => 'unreachable');
		expect(await chat.sendMessage(down, 's1', 'x')).toEqual({ ok: false, status: 503, message: '' });
	});

	it('refuses an empty message without calling the backend', async () => {
		const { backend, event } = eventWith(() => json({}));
		expect(await chat.sendMessage(event, 's1', '   ')).toMatchObject({ ok: false, status: 400 });
		expect(backend.calls).toHaveLength(0);
	});

	it('starts a chat with its first message, on the model picked for it', async () => {
		const { backend, event } = eventWith((call) => (call.url.endsWith('/api/sessions') ? json({ session_id: 'new' }, 201) : json({ user_message: message('u'), model: {} })));
		const started = await chat.startChat(event, { text: 'hi', model: 'admin:1', thinking: 'high' });
		expect(started).toEqual({ ok: true, data: { sessionId: 'new', sent: true } });
		expect(backend.calls.map((call) => `${call.method} ${call.url.replace('http://api.test', '')}`)).toEqual([
			'POST /api/sessions',
			'PUT /api/sessions/new/model',
			'PUT /api/sessions/new/thinking',
			'POST /api/sessions/new/messages',
		]);
	});

	it('still opens a new chat whose first message did not go', async () => {
		const { event } = eventWith((call) => (call.url.endsWith('/api/sessions') ? json({ session_id: 'new' }, 201) : new Response('', { status: 500 })));
		expect(await chat.startChat(event, { text: 'hi' })).toEqual({ ok: true, data: { sessionId: 'new', sent: false } });
	});

	it('renames, pins and moves a chat in one call shape', async () => {
		const { backend, event } = eventWith(() => json({ title: 'Rent', pinned: true, in_project_id: 'p1' }));
		expect(await chat.updateSession(event, 's1', { title: '  Rent ' })).toMatchObject({ ok: true, data: { title: 'Rent' } });
		await chat.updateSession(event, 's1', { pinned: true });
		await chat.updateSession(event, 's1', { in_project_id: null });
		expect(backend.calls.map((call) => call.body)).toEqual([{ title: 'Rent' }, { pinned: true }, { in_project_id: null }]);
		expect(await chat.updateSession(event, 's1', { title: ' ' })).toMatchObject({ ok: false, status: 400 });
	});

	it('rates a reply, with reasons and a note for a dislike, or takes the rating back', async () => {
		const { backend, event } = eventWith(() => new Response(null, { status: 204 }));
		await chat.setFeedback(event, 's1', 'm1', { rating: 'down', reasons: ['inaccurate'], note: ' off ', includeChat: true });
		await chat.setFeedback(event, 's1', 'm1', { rating: null });
		expect(backend.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/sessions/s1/messages/m1/feedback', body: { rating: 'down', reasons: ['inaccurate'], note: 'off', include_chat: true } });
		expect(backend.calls[1]).toMatchObject({ method: 'DELETE' });
	});

	it('runs the latest turn again: regenerated in a style, or edited', async () => {
		const done = { message_id: 'u1', version: 2, total: 2, edited_at: null };
		const { backend, event } = eventWith(() => json(done, 202));
		expect(await chat.regenerate(event, 's1', 'm1', 'shorter')).toEqual({ ok: true, data: done });
		await chat.editMessage(event, 's1', 'u1', 'new text');
		expect(backend.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/sessions/s1/messages/m1/regenerate', body: { style: 'shorter' } });
		expect(backend.calls[1]).toMatchObject({ method: 'PATCH', url: 'http://api.test/api/sessions/s1/messages/u1', body: { text: 'new text' } });
	});

	it('renders the replies of earlier versions', async () => {
		const { event } = eventWith(() => json({ message_id: 'u1', total: 2, versions: [{ version: 1, text: 'old', created_at: '', replies: [{ content: '*x*', agent_display_name: 'Nomi', created_at: '' }] }] }));
		const versions = await chat.messageVersions(event, 's1', 'u1');
		expect(versions.ok && versions.data[0].replies[0].content_html).toContain('<em>x</em>');
	});

	it('lists the projects a chat can move into, with their workspace (the chat stays private)', async () => {
		const { event } = eventWith((call) =>
			call.url.endsWith('/api/projects') ? json({ org_id: 'o2', projects: [{ id: 'p1', name: 'budget-keluarga' }] }) : json({ orgs: [{ id: 'o2', name: 'Keluarga', members: 3 }] }),
		);
		expect(await chat.moveTargets(event)).toEqual([{ id: 'p1', name: 'budget-keluarga', workspace: 'Keluarga' }]);
	});

	it('lists the chats for the Chats page, and tells a list it could not read from an empty one', async () => {
		const sessions = [{ id: 's1', title: 'Rent', pinned: false, in_project_id: null, project_id: null, last_message: null, updated_at: '' }];
		expect(await chat.chatList(eventWith(() => json({ sessions })).event)).toEqual(sessions);
		expect(await chat.chatList(eventWith(() => json({ sessions: [] })).event)).toEqual([]);
		expect(await chat.chatList(eventWith(() => new Response('', { status: 502 })).event)).toBeNull();
	});

	it("reads what Home's Today strip sums up, and has nothing to show when it can't", async () => {
		const summary = { since: '', timezone: 'Asia/Jakarta', while_you_were_out: [], today: [], plans: [], while_you_were_out_total: 0, today_total: 0, plans_total: 0 };
		const { backend, event } = eventWith(() => json(summary));
		expect(await chat.homeSummary(event)).toEqual(summary);
		expect(backend.calls[0].url).toBe('http://api.test/api/home');
		expect(await chat.homeSummary(eventWith(() => new Response('', { status: 500 })).event)).toBeNull();
	});

	it('lets a sign-in that ran out through as the redirect it is', async () => {
		const { event } = eventWith(() => new Response('', { status: 401 }));
		expect(await thrown(() => chat.sendMessage(event, 's1', 'x'))).toMatchObject({ status: 303, location: '/sign-in' });
	});
	it('names a teammate’s message fetched on its own, as the list does (the single read doesn’t)', async () => {
		const { backend, event } = eventWith((call) =>
			call.url.includes('/messages/u1') ? json(message('u1', { sender: 'user' })) : json({ messages: [message('u0', { sender: 'user' }), message('u1', { sender: 'user', author_name: 'Sari Wulandari' })] }),
		);
		const result = await chat.message(event, 's1', 'u1');
		expect(result.ok && result.data.author_name).toBe('Sari Wulandari');
		expect(backend.calls.map((call) => call.url.replace('http://api.test', ''))).toEqual(['/api/sessions/s1/messages/u1', '/api/sessions/s1/messages?limit=20']);
		// A reply, or the person's own message, costs no second read.
		const reply = eventWith(() => json(message('a1')));
		await chat.message(reply.event, 's1', 'a1');
		expect(reply.backend.calls).toHaveLength(1);
		const mine = eventWith((call) => (call.url.includes('/messages/u2') ? json(message('u2', { sender: 'user' })) : json({ messages: [message('u2', { sender: 'user' })] })));
		const own = await chat.message(mine.event, 's1', 'u2');
		expect(own.ok && own.data.author_name).toBeUndefined();
	});
});
