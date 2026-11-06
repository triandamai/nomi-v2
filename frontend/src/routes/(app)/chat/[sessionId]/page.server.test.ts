import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json, settle as settleThrow, textResponse } from '../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions, load } = await import('./+page.server');

const calls = () => apiFetch.mock.calls.map(([, , path, init]) => ({ path, method: init?.method ?? 'GET', body: init?.body ? JSON.parse(init.body) : undefined }));
const status = (result: unknown) => (result as { status?: number }).status;
const chat = (form?: Record<string, string | string[]>) => event({ form, params: { sessionId: 's1' } });

function backend(override: (path: string) => Response | undefined = () => undefined) {
	apiFetch.mockImplementation(async (_f: unknown, _c: unknown, path: string) => {
		const custom = override(path);
		if (custom) return custom;
		if (path.startsWith('/api/sessions/s1/messages?')) return json({ messages: [{ id: 'm1', sender: 'user', content: '**hi**', content_blocks: null, created_at: '', my_feedback: null, agent_display_name: null, memory_count: 0 }], has_more: true });
		if (path === '/api/personality/history') return json({ versions: [{ version: 1 }] });
		if (path === '/api/sessions') return json({ sessions: [{ id: 's1', title: 'Budget', pinned: true, in_project_id: 'p1' }] });
		if (path === '/api/projects') return json({ org_id: 'o1', projects: [{ id: 'p1', name: 'Bakery' }] });
		if (path === '/api/orgs') return json({ orgs: [{ id: 'o1', name: 'Acme' }] });
		return json({}, 404);
	});
}

beforeEach(() => {
	apiFetch.mockReset();
	backend();
});

describe('chat: load', () => {
	it('loads the chat with its title, menu and rendered messages', async () => {
		const data = (await load(chat())) as Record<string, any>;
		expect(data.title).toBe('Budget');
		expect(data.hasOlder).toBe(true);
		expect(data.messages[0].content_html).toContain('<strong>hi</strong>');
		expect(data.chatMenu).toEqual({ pinned: true, inProjectId: 'p1', projects: [{ id: 'p1', name: 'Bakery', workspace: 'Acme' }] });
		expect(data.chatPersonality).toEqual({ chat: null, default: null });
		expect(data.agentActivity).toEqual([]);
		expect(data.agentStatus).toBeNull();
	});

	it('sends a missing chat home and fails on any other error', async () => {
		backend((path) => (path.startsWith('/api/sessions/s1/messages') ? json({}, 404) : undefined));
		expect(await settleThrow(() => load(chat()))).toMatchObject({ thrown: { status: 303, location: '/' } });
		backend((path) => (path.startsWith('/api/sessions/s1/messages') ? json({}, 500) : undefined));
		expect(await settleThrow(() => load(chat()))).toMatchObject({ thrown: { status: 500 } });
	});

	it('names an unknown chat “new chat” and copes without its menu data', async () => {
		backend((path) => (path === '/api/sessions' || path === '/api/projects' || path === '/api/orgs' || path === '/api/personality/history' ? json({}, 500) : undefined));
		const data = (await load(chat())) as Record<string, any>;
		expect(data.title).toBeTruthy();
		expect(data.chatMenu).toEqual({ pinned: false, inProjectId: null, projects: [] });
		expect(data.personality).toEqual({ versions: [] });
	});
});

describe('chat: actions', () => {
	it('sends a message, refusing an empty one', async () => {
		expect(status(await actions.sendMessage(chat({ text: '  ' })))).toBe(400);
		backend(() => json({ user_message: { id: 'm2' } }));
		expect(await actions.sendMessage(chat({ text: 'hi' }))).toEqual({ user_message: { id: 'm2' }, stopped: false });
		backend(() => json({ user_message: { id: 'm3' }, supervisor_reply: { id: 'm4' } }));
		expect(await actions.sendMessage(chat({ text: 'stop' }))).toMatchObject({ stopped: true });
		backend(() => json({}, 404));
		expect(await settleThrow(() => actions.sendMessage(chat({ text: 'hi' })))).toMatchObject({ thrown: { status: 303 } });
		backend(() => json({}, 429));
		expect(status(await actions.sendMessage(chat({ text: 'hi' })))).toBe(429);
	});

	it('records feedback, with a thumbs-down’s reasons, or clears it', async () => {
		backend(() => json({}));
		expect(status(await actions.feedback(chat({})))).toBe(400);
		await actions.feedback(chat({ messageId: 'm1', rating: 'down', reasons: ['wrong', ''], note: ' too long ', includeChat: 'true' }));
		expect(calls()[0]).toEqual({ path: '/api/sessions/s1/messages/m1/feedback', method: 'PUT', body: { rating: 'down', reasons: ['wrong'], note: 'too long', include_chat: true } });
		await actions.feedback(chat({ messageId: 'm1', rating: '' }));
		expect(calls()[1].method).toBe('DELETE');
		backend(() => json({}, 500));
		expect(status(await actions.feedback(chat({ messageId: 'm1', rating: 'up' })))).toBe(500);
	});

	it('resolves approvals', async () => {
		backend(() => json({}));
		expect(status(await actions.resolveApproval(chat({ messageId: 'm1', decision: 'maybe' })))).toBe(400);
		expect(await actions.resolveApproval(chat({ messageId: 'm1', decision: 'approve', remember: 'true' }))).toEqual({ success: true });
		expect(calls()[0].body).toEqual({ decision: 'approve', remember: true });
		backend(() => textResponse('already decided', 409));
		expect(await actions.resolveApproval(chat({ messageId: 'm1', decision: 'deny' }))).toMatchObject({ status: 409, data: { error: 'already decided' } });
	});

	it('deletes the chat', async () => {
		backend(() => json({}));
		expect(await actions.deleteSession(chat())).toEqual({ deleted: true });
		backend(() => json({}, 403));
		expect(status(await actions.deleteSession(chat()))).toBe(403);
	});

	it('changes, resets and restores the personality', async () => {
		backend(() => json({}));
		expect(status(await actions.usePersonality(chat({ description: ' ' })))).toBe(400);
		expect(await actions.usePersonality(chat({ description: 'Calm', scope: 'all_chats' }))).toEqual({ personalityRestored: true });
		expect(calls()[0].body).toEqual({ description: 'Calm', scope: 'all_chats' });
		expect(await actions.resetPersonality(chat())).toEqual({ personalityRestored: true });
		expect(status(await actions.restorePersonality(chat({ version: '' })))).toBe(400);
		expect(await actions.restorePersonality(chat({ version: '3' }))).toEqual({ personalityRestored: true });
		expect(calls().at(-1)?.body).toEqual({ version: 3 });
		backend(() => textResponse('', 500));
		for (const name of ['usePersonality', 'resetPersonality', 'restorePersonality'] as const) {
			expect(status(await actions[name](chat({ description: 'x', version: '1' })))).toBe(500);
		}
	});
});
