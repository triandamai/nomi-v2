import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const shown: { message: string; actionLabel: string; href: string }[] = [];
vi.mock('$lib/snackbars.svelte', () => ({ showSnackbar: (item: { message: string; actionLabel: string; href: string }) => shown.push(item) }));

class FakeSocket {
	static all: FakeSocket[] = [];
	listeners: Record<string, ((event: { data?: string }) => void)[]> = {};
	closed = false;
	constructor(public url: string) {
		FakeSocket.all.push(this);
	}
	addEventListener(type: string, fn: (event: { data?: string }) => void) {
		(this.listeners[type] ??= []).push(fn);
	}
	close() {
		this.closed = true;
	}
	emit(type: string, data?: unknown) {
		for (const fn of this.listeners[type] ?? []) fn({ data: data === undefined ? undefined : JSON.stringify(data) });
	}
}

let activity: { busy: boolean; replied: boolean; waiting: string | null; agent: string | null };
const posted: unknown[] = [];
const fetchMock = vi.fn(async () => ({ ok: true, json: async () => activity }));

// Compiling the module (and its translations) on the real clock: the fake one stalls it.
beforeAll(async () => {
	await import('./backgroundChats');
}, 30_000);

beforeEach(() => {
	vi.useFakeTimers();
	vi.resetModules();
	shown.length = 0;
	posted.length = 0;
	FakeSocket.all = [];
	fetchMock.mockClear();
	activity = { busy: false, replied: true, waiting: null, agent: 'Finley' };
	const store = new Map<string, string>();
	vi.stubGlobal('WebSocket', FakeSocket);
	vi.stubGlobal('fetch', fetchMock);
	vi.stubGlobal('sessionStorage', { getItem: (k: string) => store.get(k) ?? null, setItem: (k: string, v: string) => store.set(k, v) });
	vi.stubGlobal('document', { hidden: false, cookie: '' });
	vi.stubGlobal('location', { pathname: '/chat/other' });
	vi.stubGlobal(
		'BroadcastChannel',
		class {
			postMessage(data: unknown) {
				posted.push(data);
			}
			addEventListener() {}
			removeEventListener() {}
		},
	);
});

afterEach(() => {
	vi.useRealTimers();
	vi.unstubAllGlobals();
});

const chat = { sessionId: 's1', href: '/chat/s1', title: 'Trip to Bandung', openTurns: ['t1'] };

async function settle(ms: number) {
	await vi.advanceTimersByTimeAsync(ms);
}

describe('a chat left while the crew works on it', () => {
	it('says when it is done, with a way back', async () => {
		const { watchChat } = await import('./backgroundChats');
		watchChat(chat);
		const socket = FakeSocket.all[0];
		expect(socket.url).toBe('/chat/s1/ws');
		socket.emit('open');
		socket.emit('message', { kind: 'Delta', turn_job_id: 't1', event: {} });
		await settle(5000);
		expect(shown).toEqual([]); // still streaming
		socket.emit('message', { kind: 'TurnCompleted', turn_job_id: 't1', message_id: 'm1' });
		await settle(3000);
		expect(shown).toEqual([expect.objectContaining({ message: 'Finley finished in “Trip to Bandung”', actionLabel: 'Open', href: '/chat/s1' })]);
		expect(socket.closed).toBe(true);
	});

	it('waits while a crew member still has the work, then asks for the answer it needs', async () => {
		const { watchChat } = await import('./backgroundChats');
		watchChat(chat);
		const socket = FakeSocket.all[0];
		socket.emit('open');
		activity = { busy: true, replied: true, waiting: null, agent: 'Nomi' };
		socket.emit('message', { kind: 'TurnCompleted', turn_job_id: 't1' });
		await settle(3000);
		expect(shown).toEqual([]);
		activity = { busy: false, replied: true, waiting: 'question', agent: 'Rena' };
		await settle(20_000);
		expect(shown).toEqual([expect.objectContaining({ message: 'Rena has a question for you in “Trip to Bandung”', actionLabel: 'Answer' })]);
	});

	it('says nothing once the person is back in the chat', async () => {
		const { watchChat, unwatchChat } = await import('./backgroundChats');
		watchChat(chat);
		const socket = FakeSocket.all[0];
		socket.emit('open');
		unwatchChat('s1');
		socket.emit('message', { kind: 'TurnCompleted', turn_job_id: 't1' });
		await settle(30_000);
		expect(shown).toEqual([]);
		expect(socket.closed).toBe(true);
	});

	it('does not call a turn that has not started yet done', async () => {
		const { watchChat } = await import('./backgroundChats');
		watchChat({ ...chat, openTurns: [] }); // sent, then left before anything streamed
		FakeSocket.all[0].emit('open');
		activity = { busy: false, replied: false, waiting: null, agent: null };
		await settle(16_000);
		expect(shown).toEqual([]);
		activity = { busy: false, replied: true, waiting: null, agent: 'Nomi' };
		await settle(20_000);
		expect(shown).toEqual([expect.objectContaining({ message: 'Nomi finished in “Trip to Bandung”' })]);
	});

	it('tells the tab the person is looking at when it happens in a hidden one', async () => {
		const { watchChat } = await import('./backgroundChats');
		watchChat(chat);
		vi.stubGlobal('document', { hidden: true, cookie: '' });
		const socket = FakeSocket.all[0];
		socket.emit('open');
		socket.emit('message', { kind: 'TurnFailed', turn_job_id: 't1', error: '' });
		activity = { busy: false, replied: false, waiting: null, agent: null };
		await settle(3000);
		expect(shown).toEqual([]);
		expect(posted).toEqual([expect.objectContaining({ message: 'Something went wrong in “Trip to Bandung”' })]);
	});

	it('picks the watching back up after a reload', async () => {
		const first = await import('./backgroundChats');
		first.watchChat(chat);
		vi.resetModules();
		FakeSocket.all = [];
		const reloaded = await import('./backgroundChats');
		reloaded.resumeWatches();
		const socket = FakeSocket.all[0];
		expect(socket.url).toBe('/chat/s1/ws');
		socket.emit('open');
		await settle(3000);
		expect(shown).toEqual([expect.objectContaining({ href: '/chat/s1' })]);
	});
});
