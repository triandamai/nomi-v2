import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { connectActivity, routeActivity, type ActivityHandlers, type Notification } from './activity';
import { FakeSocket } from './fakeSocket';

// The person's own live feed (backend: nomi_realtime::UserEnvelope, relayed at /activity/ws),
// ported from the old app's userActivity (frontend/src/lib/userActivity.ts).

function handlers() {
	return {
		chatsChanged: vi.fn<(sessionId: string) => void>(),
		notification: vi.fn<(notification: Notification) => void>(),
		signedOutHere: vi.fn<() => void>(),
		resumed: vi.fn<() => void>(),
		lost: vi.fn<(code: number) => void>(),
	} satisfies ActivityHandlers;
}

describe('routeActivity', () => {
	it('refreshes the chat lists when a chat moved on, changed or went away', () => {
		const on = handlers();
		routeActivity({ kind: 'ChatActivity', session_id: 'a', title: 'Budget', project_id: null, outcome: 'done', waiting: null, agent: 'money' }, 'dev-1', on);
		routeActivity({ kind: 'ChatUpdated', session_id: 'b', project_id: null, title: null }, 'dev-1', on);
		routeActivity({ kind: 'ChatDeleted', session_id: 'c' }, 'dev-1', on);
		expect(on.chatsChanged.mock.calls).toEqual([['a'], ['b'], ['c']]);
	});

	it('hands a new inbox item over', () => {
		const on = handlers();
		routeActivity({ kind: 'NotificationCreated', notification_id: 'n1', category: 'team', title: 'Andi invited you', body: 'Join Kopi Kita', link: '/invite/abc' }, null, on);
		expect(on.notification).toHaveBeenCalledWith({ id: 'n1', title: 'Andi invited you', body: 'Join Kopi Kita', link: '/invite/abc' });
	});

	it('signs out here only when this device was signed out', () => {
		const on = handlers();
		routeActivity({ kind: 'DeviceSignedOut', device_id: 'OTHER' }, 'dev-1', on);
		expect(on.signedOutHere).not.toHaveBeenCalled();
		routeActivity({ kind: 'DeviceSignedOut', device_id: 'DEV-1' }, 'dev-1', on);
		expect(on.signedOutHere).toHaveBeenCalledOnce();
	});

	it('ignores what it doesn’t know', () => {
		const on = handlers();
		routeActivity({ kind: 'DevicePresence', device_id: 'x', presence: 'online', last_seen_at: '' }, null, on);
		routeActivity({ kind: 'Something new' }, null, on);
		routeActivity('not an event', null, on);
		expect([on.chatsChanged, on.notification, on.signedOutHere].every((fn) => fn.mock.calls.length === 0)).toBe(true);
	});
});

describe('connectActivity', () => {
	let sockets: FakeSocket[];
	let urls: string[];
	const visibility = { hidden: false, listeners: [] as (() => void)[] };

	beforeEach(() => {
		vi.useFakeTimers();
		sockets = [];
		urls = [];
		visibility.hidden = false;
		visibility.listeners = [];
	});
	afterEach(() => vi.useRealTimers());

	function connect(on = handlers()) {
		const stop = connectActivity({
			deviceId: 'dev-1',
			tab: 'tab12345',
			handlers: on,
			page: { hidden: () => visibility.hidden, onVisibilityChange: (listener) => (visibility.listeners.push(listener), () => (visibility.listeners = visibility.listeners.filter((l) => l !== listener))) },
			openSocket: (url) => {
				urls.push(url);
				const socket = new FakeSocket();
				sockets.push(socket);
				return socket;
			},
		});
		return { stop, on };
	}

	it('opens this tab’s feed and routes its events', () => {
		const { on } = connect();
		expect(urls).toEqual(['/activity/ws?tab=tab12345']);
		sockets[0].fire('open');
		sockets[0].fire('message', { data: JSON.stringify({ kind: 'ChatDeleted', session_id: 'c' }) });
		sockets[0].fire('message', { data: 'not json' });
		expect(on.chatsChanged).toHaveBeenCalledWith('c');
	});

	it('reloads what it may have missed after coming back (events aren’t replayed)', () => {
		const { on } = connect();
		sockets[0].fire('open');
		expect(on.resumed).not.toHaveBeenCalled();
		sockets[0].fire('close', { code: 1006 });
		vi.advanceTimersByTime(1250);
		sockets[1].fire('open');
		expect(on.resumed).toHaveBeenCalledOnce();
	});

	it('says when the sign-in is over', () => {
		const { on } = connect();
		sockets[0].fire('close', { code: 4401 });
		expect(on.lost).toHaveBeenCalledWith(4401);
	});

	it('tells the backend whether the tab is shown', () => {
		visibility.hidden = true;
		const { stop } = connect();
		sockets[0].fire('open');
		expect(sockets[0].sent).toEqual(['{"op":"presence","state":"away"}']);
		visibility.hidden = false;
		visibility.listeners.forEach((listener) => listener());
		expect(sockets[0].sent.at(-1)).toBe('{"op":"presence","state":"online"}');
		stop();
		expect(visibility.listeners).toHaveLength(0);
		expect(sockets[0].closed).toBe(true);
	});
});
