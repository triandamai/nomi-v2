// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync, tick } from 'svelte';
import { click, control, controls, render, text, type } from '../../test-stubs/render';
import type { RenderedMessage } from '$lib/types';

const nav = vi.hoisted(() => ({ goto: vi.fn(async () => {}), invalidateAll: vi.fn(async () => {}) }));
vi.mock('$app/navigation', () => ({
	goto: nav.goto,
	invalidateAll: nav.invalidateAll,
	afterNavigate: () => {},
	beforeNavigate: () => {},
}));

const ChatThread = (await import('./ChatThread.svelte')).default;

/** Stand-in for the chat's WebSocket: the test opens it and pushes the server's events. */
class FakeSocket {
	static all: FakeSocket[] = [];
	listeners: Record<string, ((event: unknown) => void)[]> = {};
	closed = false;
	constructor(public url: string) {
		FakeSocket.all.push(this);
	}
	addEventListener(type: string, listener: (event: unknown) => void) {
		(this.listeners[type] ??= []).push(listener);
	}
	close() {
		this.closed = true;
	}
	emit(type: string, event: unknown = {}) {
		for (const listener of this.listeners[type] ?? []) listener(event);
		flushSync();
	}
	send(envelope: Record<string, unknown>) {
		this.emit('message', { data: JSON.stringify(envelope) });
	}
}

const message = (over: Partial<RenderedMessage> = {}): RenderedMessage => ({
	id: 'm1',
	sender: 'user',
	content: 'How much did I spend?',
	content_html: '<p>How much did I spend?</p>',
	content_blocks: null,
	created_at: '2026-11-05T09:00:00Z',
	my_feedback: null,
	agent_display_name: null,
	memory_count: 0,
	...over,
});

const reply = message({
	id: 'm2',
	sender: 'assistant',
	content: 'Rp 1.2 juta this month.',
	content_html: '<p>Rp 1.2 juta this month.</p>',
	agent_display_name: 'Finley',
	created_at: '2026-11-05T09:00:05Z',
	memory_count: 2,
});

const fetchMock = vi.fn();

beforeEach(() => {
	FakeSocket.all = [];
	vi.stubGlobal('WebSocket', FakeSocket);
	fetchMock.mockReset();
	fetchMock.mockImplementation(async (url: string) =>
		String(url).includes('/message/m3')
			? new Response(JSON.stringify(message({ id: 'm3', sender: 'assistant', content: 'Rp 900 ribu', content_html: '<p>Rp 900 ribu so far.</p>', created_at: '2026-11-05T09:01:00Z' })), { status: 200 })
			: new Response(JSON.stringify({ type: 'success', status: 200, data: '[]' }), { status: 200 }),
	);
	vi.stubGlobal('fetch', fetchMock);
	nav.goto.mockClear();
	nav.invalidateAll.mockClear();
});

afterEach(() => {
	vi.unstubAllGlobals();
});

function thread(props: Record<string, unknown> = {}) {
	return render(ChatThread as never, {
		sessionId: 's1',
		messages: [message(), reply],
		agentActivity: [],
		title: 'Spending check',
		...props,
	} as never);
}

describe('ChatThread', () => {
	it('shows the conversation and follows the chat on its live feed', () => {
		const root = thread();
		expect(text(root)).toContain('How much did I spend?');
		expect(text(root)).toContain('Rp 1.2 juta this month.');
		expect(text(root)).toContain('Spending check');
		expect(FakeSocket.all).toHaveLength(1);
		expect(FakeSocket.all[0].url).toBe('/chat/s1/ws');
	});

	it('does not connect a chat that is not saved yet', () => {
		thread({ unsaved: true, messages: [] });
		expect(FakeSocket.all).toHaveLength(0);
	});

	it('syncs once on open, and again when the feed comes back after a drop', () => {
		thread();
		const socket = FakeSocket.all[0];
		socket.emit('open');
		expect(nav.invalidateAll).toHaveBeenCalledTimes(1);
	});

	it('streams a reply, then fetches the posted message when the turn completes', async () => {
		const root = thread();
		const socket = FakeSocket.all[0];
		socket.send({ kind: 'AgentPhaseChanged', turn_job_id: 't1', phase: 'thinking', detail: 'Reading your money' });
		socket.send({ kind: 'Delta', turn_job_id: 't1', event: { type: 'text', text: 'Rp 900' } });
		socket.send({ kind: 'Delta', turn_job_id: 't1', event: { type: 'TextDelta', delta: 'Rp 900 ribu' } });
		expect(root.querySelector('.chat')).not.toBeNull();
		socket.send({ kind: 'TurnCompleted', turn_job_id: 't1', message_id: 'm3' });
		await vi.waitFor(() => expect(text(root)).toContain('Rp 900 ribu so far.'));
		expect(fetchMock.mock.calls.some(([url]) => String(url).includes('m3'))).toBe(true);
	});

	it('drops deleted messages and renames from the feed', () => {
		const root = thread();
		const socket = FakeSocket.all[0];
		socket.send({ kind: 'MessagesDeleted', message_ids: ['m2'] });
		expect(text(root)).not.toContain('Rp 1.2 juta this month.');
		socket.send({ kind: 'SessionRenamed', title: 'Budget talk' });
		expect(text(root)).toContain('Budget talk');
		socket.send({ kind: 'PersonalityChanged' });
		socket.send({ kind: 'AgentDelegationUpdated' });
		expect(nav.invalidateAll).toHaveBeenCalled();
	});

	it('ignores feed messages that are not JSON', () => {
		const root = thread();
		FakeSocket.all[0].emit('message', { data: 'not json' });
		expect(text(root)).toContain('How much did I spend?');
	});

	it('leaves the chat when it is deleted elsewhere', () => {
		thread();
		FakeSocket.all[0].send({ kind: 'SessionDeleted' });
		expect(nav.goto).toHaveBeenCalledWith('/chats', { replaceState: true });
		expect(FakeSocket.all[0].closed).toBe(true);
	});

	it('shows a failed turn that came without a reason', () => {
		const root = thread();
		const socket = FakeSocket.all[0];
		socket.send({ kind: 'Delta', turn_job_id: 't1', event: { type: 'text', text: 'Half' } });
		socket.send({ kind: 'TurnFailed', turn_job_id: 't1' });
		expect(root.querySelector('.chat')).not.toBeNull();
		socket.send({ kind: 'Delta', turn_job_id: 't2', event: { type: 'text', text: 'Half' } });
		socket.send({ kind: 'TurnFailed', turn_job_id: 't2', error: 'Out of credits' });
	});

	it('says the session ended when the server closes the feed for good', () => {
		const root = thread();
		FakeSocket.all[0].emit('close', { code: 4401 });
		expect(root.querySelector('.chat')).not.toBeNull();
	});

	it('renames the chat in place', async () => {
		const root = thread();
		click(root.querySelector('.appbar__title-button'));
		const field = root.querySelector<HTMLInputElement>('.appbar input');
		expect(field).not.toBeNull();
	});

	it('types into the composer and opens the crew sheet', () => {
		const root = thread({ agentStatus: null, thinkingLevel: 'low' });
		const input = root.querySelector('textarea');
		type(input, 'And last month?');
		expect((input as HTMLTextAreaElement).value).toBe('And last month?');
		click(root.querySelector('.appbar__crew'));
		expect(text()).toBeTruthy();
	});

	it('shows a project chat with its status and a read-only note', () => {
		const root = thread({ project: { status: 'building' }, readOnlyNote: 'Only editors can write here.', compactHeader: true });
		expect(text(root)).toContain('Only editors can write here.');
		expect(root.querySelector('.chat--project')).not.toBeNull();
	});

	it('shows a send error and an older-messages loader', () => {
		const root = thread({ sendError: 'Could not send', hasOlder: true, glass: true, typingLabel: 'Ben is typing…', typingPeople: [{ name: 'Ben', initials: 'B' }] });
		expect(text(root)).toContain('Could not send');
	});

	it('opens the menu from the app bar on a phone', () => {
		const onopenmenu = vi.fn();
		thread({ onopenmenu });
		click(controls(/menu/i)[0]);
		expect(onopenmenu).toHaveBeenCalled();
	});

	it('closes the feed when the chat closes', () => {
		thread();
		expect(control(/./).isConnected).toBe(true);
	});
});
