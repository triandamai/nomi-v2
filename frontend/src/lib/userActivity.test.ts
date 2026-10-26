import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const shown: { message: string; actionLabel: string; href: string }[] = [];
vi.mock('$lib/snackbars.svelte', () => ({ showSnackbar: (item: { message: string; actionLabel: string; href: string }) => shown.push(item) }));

class FakeSocket {
	static all: FakeSocket[] = [];
	listeners: Record<string, ((event: { data?: string; code?: number }) => void)[]> = {};
	closed = false;
	constructor(public url: string) {
		FakeSocket.all.push(this);
	}
	addEventListener(type: string, fn: (event: { data?: string; code?: number }) => void) {
		(this.listeners[type] ??= []).push(fn);
	}
	close() {
		this.closed = true;
	}
	emit(type: string, event: { data?: string; code?: number } = {}) {
		for (const fn of this.listeners[type] ?? []) fn(event);
	}
}

const activity = (over: Record<string, unknown> = {}) =>
	JSON.stringify({ kind: 'ChatActivity', session_id: 's1', title: 'Trip to Bandung', project_id: null, outcome: 'done', waiting: null, agent: 'Finley', ...over });

// Compiling the module (and its translations) before the fake clock starts.
beforeAll(async () => {
	await import('./userActivity');
}, 30_000);

beforeEach(() => {
	vi.useFakeTimers();
	shown.length = 0;
	FakeSocket.all = [];
	vi.stubGlobal('WebSocket', FakeSocket);
	vi.stubGlobal('document', { hidden: false, cookie: '' });
	vi.stubGlobal('location', { pathname: '/chat/other' });
});

afterEach(() => {
	vi.useRealTimers();
	vi.unstubAllGlobals();
});

describe("the person's activity feed", () => {
	it('turns a finished chat into a snackbar with a way back', async () => {
		const { connectUserActivity } = await import('./userActivity');
		const close = connectUserActivity();
		const socket = FakeSocket.all[0];
		expect(socket.url).toBe('/activity/ws');
		socket.emit('message', { data: activity() });
		expect(shown).toEqual([expect.objectContaining({ message: 'Finley finished in “Trip to Bandung”', actionLabel: 'Open', href: '/chat/s1' })]);
		close();
		expect(socket.closed).toBe(true);
	});

	it("asks for what's waiting, and goes to a project's own page", async () => {
		const { connectUserActivity } = await import('./userActivity');
		connectUserActivity();
		const socket = FakeSocket.all[0];
		socket.emit('message', { data: activity({ outcome: 'waiting', waiting: 'question', agent: 'Rena', project_id: 'p1', title: 'Landing page' }) });
		socket.emit('message', { data: activity({ outcome: 'waiting', waiting: 'approval', agent: 'Tara' }) });
		expect(shown).toEqual([
			expect.objectContaining({ message: 'Rena has a question for you in “Landing page”', actionLabel: 'Answer', href: '/projects/session/s1' }),
			expect.objectContaining({ message: 'Tara needs your OK in “Trip to Bandung”', actionLabel: 'Review' }),
		]);
	});

	it('stays quiet about the chat on screen, in a background tab, and for unknown events', async () => {
		const { connectUserActivity } = await import('./userActivity');
		connectUserActivity();
		const socket = FakeSocket.all[0];
		vi.stubGlobal('location', { pathname: '/chat/s1' });
		socket.emit('message', { data: activity() });
		vi.stubGlobal('location', { pathname: '/' });
		vi.stubGlobal('document', { hidden: true, cookie: '' });
		socket.emit('message', { data: activity() });
		vi.stubGlobal('document', { hidden: false, cookie: '' });
		socket.emit('message', { data: JSON.stringify({ kind: 'SomethingNew' }) });
		socket.emit('message', { data: 'not json' });
		expect(shown).toEqual([]);
	});

	it('tells chat lists when a chat moved on elsewhere, without a snackbar', async () => {
		const { connectUserActivity, onChatsChanged } = await import('./userActivity');
		connectUserActivity();
		const changed: string[] = [];
		const stop = onChatsChanged((sessionId) => changed.push(sessionId));
		const socket = FakeSocket.all[0];
		socket.emit('message', { data: JSON.stringify({ kind: 'ChatUpdated', session_id: 's2', project_id: null, title: null }) });
		socket.emit('message', { data: activity() });
		stop();
		socket.emit('message', { data: JSON.stringify({ kind: 'ChatUpdated', session_id: 's3', project_id: null, title: null }) });
		expect(changed).toEqual(['s2', 's1']);
		expect(shown).toHaveLength(1);
	});

	it('reconnects when the feed drops, but not once signed out', async () => {
		const { connectUserActivity } = await import('./userActivity');
		connectUserActivity();
		FakeSocket.all[0].emit('close', { code: 1006 });
		await vi.advanceTimersByTimeAsync(1500);
		expect(FakeSocket.all).toHaveLength(2);
		FakeSocket.all[1].emit('close', { code: 4401 });
		await vi.advanceTimersByTimeAsync(60_000);
		expect(FakeSocket.all).toHaveLength(2);
	});
});
