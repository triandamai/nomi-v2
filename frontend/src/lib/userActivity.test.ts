import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const shown: { message: string; actionLabel: string; href: string }[] = [];
const arrived: { id: string; title: string; body?: string; link: string | null }[] = [];
vi.mock('$lib/notifications.svelte', () => ({ notificationArrived: (item: (typeof arrived)[number]) => arrived.push(item) }));
vi.mock('$lib/snackbars.svelte', () => ({ showSnackbar: (item: { message: string; actionLabel: string; href: string }) => shown.push(item) }));

class FakeSocket {
	static all: FakeSocket[] = [];
	listeners: Record<string, ((event: { data?: string; code?: number }) => void)[]> = {};
	closed = false;
	readyState = 1;
	sent: string[] = [];
	static OPEN = 1;
	constructor(public url: string) {
		FakeSocket.all.push(this);
	}
	addEventListener(type: string, fn: (event: { data?: string; code?: number }) => void) {
		(this.listeners[type] ??= []).push(fn);
	}
	close() {
		this.closed = true;
	}
	send(frame: string) {
		this.sent.push(frame);
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
	it('hands a new inbox item, with what happened, to the inbox', async () => {
		const { connectUserActivity } = await import('./userActivity');
		arrived.length = 0;
		connectUserActivity();
		const notification = {
			kind: 'NotificationCreated',
			notification_id: 'n1',
			category: 'deploy',
			title: "Shop's backend couldn't be set up",
			body: 'Shipyard refused it: plan limit reached (3 environments).',
			link: '/projects/p1/app?sheet=deploy',
		};
		FakeSocket.all[0].emit('message', { data: JSON.stringify(notification) });
		expect(arrived).toEqual([{ id: 'n1', title: notification.title, body: notification.body, link: notification.link }]);
		expect(shown).toEqual([]);
	});

	it('turns a finished chat into a snackbar with a way back', async () => {
		const { connectUserActivity } = await import('./userActivity');
		const close = connectUserActivity();
		const socket = FakeSocket.all[0];
		expect(socket.url).toMatch(/^\/activity\/ws\?tab=[a-z0-9]+$/);
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

	it('tells chat lists when a chat was deleted on another device', async () => {
		const { connectUserActivity, onChatsChanged } = await import('./userActivity');
		connectUserActivity();
		const changed: string[] = [];
		onChatsChanged((sessionId) => changed.push(sessionId));
		FakeSocket.all[0].emit('message', { data: JSON.stringify({ kind: 'ChatDeleted', session_id: 's4' }) });
		expect(changed).toEqual(['s4']);
		expect(shown).toEqual([]);
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

	it("says when the tab is hidden and shown again, on the tab's own connection", async () => {
		const visibility: (() => void)[] = [];
		const page = { hidden: false, cookie: '', addEventListener: (_: string, fn: () => void) => visibility.push(fn), removeEventListener: () => visibility.pop() };
		vi.stubGlobal('document', page);
		const { connectUserActivity } = await import('./userActivity');
		const close = connectUserActivity('d1');
		const socket = FakeSocket.all[0];
		socket.emit('open');
		expect(socket.sent).toEqual([]); // a shown tab is online from the start

		page.hidden = true;
		visibility.forEach((fn) => fn());
		page.hidden = false;
		visibility.forEach((fn) => fn());
		expect(socket.sent.map((frame) => JSON.parse(frame))).toEqual([
			{ op: 'presence', state: 'away' },
			{ op: 'presence', state: 'online' },
		]);
		close();
		expect(visibility).toHaveLength(0);
	});

	it("tells the device list when one of the person's devices comes and goes", async () => {
		const { connectUserActivity, onDevicePresence } = await import('./userActivity');
		connectUserActivity('d1');
		const seen: string[] = [];
		const stop = onDevicePresence((event) => seen.push(`${event.device_id}:${event.presence}`));
		FakeSocket.all[0].emit('message', { data: JSON.stringify({ kind: 'DevicePresence', device_id: 'd2', presence: 'away', last_seen_at: '2026-11-01T10:00:00+00:00' }) });
		stop();
		expect(seen).toEqual(['d2:away']);
		expect(shown).toEqual([]);
	});

	it('leaves when this browser was signed out from another device, and only then', async () => {
		const posted: string[] = [];
		vi.stubGlobal('fetch', async (url: string) => {
			posted.push(url);
			return new Response(null);
		});
		const assigned: string[] = [];
		vi.stubGlobal('location', { pathname: '/chat/other', assign: (to: string) => assigned.push(to) });
		const { connectUserActivity } = await import('./userActivity');
		connectUserActivity('D1-THIS');
		const socket = FakeSocket.all[0];
		socket.emit('message', { data: JSON.stringify({ kind: 'DeviceSignedOut', device_id: 'someone-else' }) });
		socket.emit('message', { data: JSON.stringify({ kind: 'DeviceSignedOut', device_id: 'd1-this' }) });
		await vi.advanceTimersByTimeAsync(0);
		expect(posted).toEqual(['/logout']);
		expect(assigned).toEqual(['/login']);
	});
});
