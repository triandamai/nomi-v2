import type { LinkSocket } from './link';

/** A WebSocket stand-in for tests: fire its events by hand, read what was sent. */
export class FakeSocket implements LinkSocket {
	closed = false;
	open = false;
	sent: string[] = [];
	#listeners: Record<string, ((event: never) => void)[]> = {};
	addEventListener(type: string, listener: (event: never) => void) {
		(this.#listeners[type] ??= []).push(listener);
	}
	send(data: string) {
		this.sent.push(data);
	}
	close() {
		this.closed = true;
	}
	get readyState() {
		return this.open ? 1 : 0;
	}
	fire(type: 'open' | 'message' | 'close', event: object = {}) {
		if (type === 'open') this.open = true;
		if (type === 'close') this.open = false;
		for (const listener of this.#listeners[type] ?? []) listener(event as never);
	}
}
