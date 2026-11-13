import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createLiveLink, type LinkEvents } from './link';
import { FakeSocket } from './fakeSocket';

// Ported from the old app's chatLink (frontend/src/lib/chatLink.ts), for one feed.

function setup(events: Partial<LinkEvents> = {}) {
	const sockets: FakeSocket[] = [];
	const opened = vi.fn<(resumed: boolean) => void>();
	const lost = vi.fn<(code: number) => void>();
	const message = vi.fn<(data: unknown) => void>();
	const link = createLiveLink({ opened: events.opened ?? opened, message: events.message ?? message, lost: events.lost ?? lost }, () => {
		const socket = new FakeSocket();
		sockets.push(socket);
		return socket;
	});
	return { link, sockets, opened, lost, message };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('createLiveLink', () => {
	it('connects at once and hands over each message', () => {
		const { sockets, opened, message } = setup();
		expect(sockets).toHaveLength(1);
		sockets[0].fire('open');
		sockets[0].fire('message', { data: '{"kind":"Hello"}' });
		expect(opened.mock.calls).toEqual([[false]]);
		expect(message).toHaveBeenCalledWith('{"kind":"Hello"}');
	});

	it('reconnects after a drop, backing off, and says it resumed', () => {
		const { sockets, opened } = setup();
		sockets[0].fire('open');
		sockets[0].fire('close', { code: 1006 });
		vi.advanceTimersByTime(999);
		expect(sockets).toHaveLength(1);
		vi.advanceTimersByTime(260);
		expect(sockets).toHaveLength(2);
		sockets[1].fire('close', { code: 1011 });
		vi.advanceTimersByTime(1250);
		expect(sockets).toHaveLength(2);
		vi.advanceTimersByTime(1000);
		expect(sockets).toHaveLength(3);
		sockets[2].fire('open');
		expect(opened.mock.calls).toEqual([[false], [true]]);
	});

	it('starts the backoff over once a connection holds', () => {
		const { sockets } = setup();
		sockets[0].fire('close', { code: 1006 });
		vi.advanceTimersByTime(1250);
		sockets[1].fire('close', { code: 1006 });
		vi.advanceTimersByTime(2250);
		sockets[2].fire('open');
		sockets[2].fire('close', { code: 1006 });
		vi.advanceTimersByTime(1250);
		expect(sockets).toHaveLength(4);
	});

	it('never waits more than 30 seconds between tries', () => {
		const { sockets } = setup();
		for (let attempt = 0; attempt < 8; attempt++) {
			sockets.at(-1)!.fire('close', { code: 1006 });
			vi.advanceTimersByTime(30_250);
		}
		expect(sockets).toHaveLength(9);
	});

	it('stops for good when the server ends it (signed out, not allowed, not found)', () => {
		for (const code of [4401, 4403, 4404]) {
			const { sockets, lost } = setup();
			sockets[0].fire('close', { code });
			vi.advanceTimersByTime(60_000);
			expect(lost).toHaveBeenCalledWith(code);
			expect(sockets).toHaveLength(1);
		}
	});

	it('closes without retrying', () => {
		const { link, sockets, lost } = setup();
		link.close();
		sockets[0].fire('close', { code: 1000 });
		vi.advanceTimersByTime(60_000);
		expect(sockets).toHaveLength(1);
		expect(sockets[0].closed).toBe(true);
		expect(lost).not.toHaveBeenCalled();
	});

	it('sends only on an open connection', () => {
		const { link, sockets } = setup();
		expect(link.send('early')).toBe(false);
		sockets[0].fire('open');
		expect(link.send('hi')).toBe(true);
		expect(sockets[0].sent).toEqual(['hi']);
	});
});
