import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createChatLink, type ChatLinkEvents, type LinkSocket } from './chatLink';

class FakeSocket implements LinkSocket {
	closed = false;
	#listeners: Record<string, ((event: never) => void)[]> = {};
	constructor(readonly sessionId: string) {}
	addEventListener(type: string, listener: (event: never) => void) {
		(this.#listeners[type] ??= []).push(listener);
	}
	close() {
		this.closed = true;
	}
	fire(type: 'open' | 'message' | 'close', event: object = {}) {
		for (const listener of this.#listeners[type] ?? []) listener(event as never);
	}
}

function setup(events: Partial<ChatLinkEvents> = {}) {
	const sockets: FakeSocket[] = [];
	const opened = vi.fn<(resumed: boolean) => void>();
	const lost = vi.fn();
	const link = createChatLink(
		{ opened: events.opened ?? opened, message: events.message ?? (() => {}), lost: events.lost ?? lost },
		(id) => {
			const socket = new FakeSocket(id);
			sockets.push(socket);
			return socket;
		},
	);
	return { link, sockets, opened, lost };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('createChatLink', () => {
	it('keeps one connection while the same chat reloads (the open sync must not loop)', () => {
		// ChatThread syncs (invalidateAll) when the socket opens; the reload hands the page a new
		// `page.params`, which re-runs the caller with the same session id.
		let syncs = 0;
		const { link, sockets } = setup({
			opened: () => {
				syncs++;
				link.follow('chat-1');
			},
		});
		link.follow('chat-1');
		// Each new socket opens once (capped, so the broken behaviour ends the test).
		for (let next = 0; next < sockets.length && next < 5; next++) sockets[next].fire('open');

		expect(sockets).toHaveLength(1);
		expect(sockets[0].closed).toBe(false);
		expect(syncs).toBe(1);
	});

	it('moves to another chat, closing the last one', () => {
		const { link, sockets } = setup();
		link.follow('chat-1');
		link.follow('chat-2');
		expect(sockets.map((s) => s.sessionId)).toEqual(['chat-1', 'chat-2']);
		expect(sockets[0].closed).toBe(true);
		expect(sockets[1].closed).toBe(false);
	});

	it('reconnects after a drop and says it resumed', () => {
		const { link, sockets, opened } = setup();
		link.follow('chat-1');
		sockets[0].fire('open');
		sockets[0].fire('close', { code: 1006 });
		vi.advanceTimersByTime(1250);
		expect(sockets).toHaveLength(2);
		sockets[1].fire('open');
		expect(opened.mock.calls).toEqual([[false], [true]]);
	});

	it('stops for good when the server ends it', () => {
		const { link, sockets, lost } = setup();
		link.follow('chat-1');
		sockets[0].fire('close', { code: 4401 });
		vi.advanceTimersByTime(60_000);
		expect(lost).toHaveBeenCalledOnce();
		expect(sockets).toHaveLength(1);
	});

	it('closes without retrying, and can follow the same chat again afterwards', () => {
		const { link, sockets } = setup();
		link.follow('chat-1');
		link.close();
		sockets[0].fire('close', { code: 1000 });
		vi.advanceTimersByTime(60_000);
		expect(sockets).toHaveLength(1);
		expect(sockets[0].closed).toBe(true);

		link.follow('chat-1');
		expect(sockets).toHaveLength(2);
	});
});
