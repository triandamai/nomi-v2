// One live connection (a feed relayed by ws-proxy): opened at once, reopened with backoff when it
// drops, given up for good when the server ends it. What the messages mean stays with the caller.
// Ported from the old app's chatLink (frontend/src/lib/chatLink.ts).

/** What the link needs from a socket: a WebSocket in the browser, a stand-in in tests. */
export interface LinkSocket {
	addEventListener(type: 'open', listener: () => void): void;
	addEventListener(type: 'message', listener: (event: { data: unknown }) => void): void;
	addEventListener(type: 'close', listener: (event: { code: number }) => void): void;
	send(data: string): void;
	close(): void;
	readonly readyState: number;
}

export interface LinkEvents {
	/** The socket opened; `resumed` when it came back after a drop rather than on the first open. */
	opened(resumed: boolean): void;
	message(data: unknown): void;
	/** The server ended it for good (signed out, not allowed, not found): no retry. */
	lost(code: number): void;
}

export interface LiveLink {
	/** Sends on the open connection; false when there's none right now. */
	send(data: string): boolean;
	close(): void;
}

/** Signed out (4401), not allowed (4403) or no such thing (4404): reconnecting won't help. */
const TERMINAL_CLOSE_CODES = new Set([4401, 4403, 4404]);
const INITIAL_RETRY_DELAY_MS = 1000;
const MAX_RETRY_DELAY_MS = 30_000;
const MAX_JITTER_MS = 250;
const OPEN = 1;

export function createLiveLink(events: LinkEvents, openSocket: () => LinkSocket): LiveLink {
	let socket: LinkSocket | undefined;
	let retryDelay = INITIAL_RETRY_DELAY_MS;
	let retryTimeout: ReturnType<typeof setTimeout> | undefined;
	let closed = false;
	let hasConnectedBefore = false;

	function connect() {
		const current = openSocket();
		socket = current;
		current.addEventListener('open', () => {
			retryDelay = INITIAL_RETRY_DELAY_MS;
			events.opened(hasConnectedBefore);
			hasConnectedBefore = true;
		});
		current.addEventListener('message', (event) => events.message(event.data));
		current.addEventListener('close', (event) => {
			if (closed || current !== socket) return;
			if (TERMINAL_CLOSE_CODES.has(event.code)) {
				events.lost(event.code);
				return;
			}
			retryTimeout = setTimeout(connect, retryDelay + Math.random() * MAX_JITTER_MS);
			retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
		});
	}

	connect();

	return {
		send(data) {
			if (socket?.readyState !== OPEN) return false;
			socket.send(data);
			return true;
		},
		close() {
			closed = true;
			clearTimeout(retryTimeout);
			socket?.close();
		},
	};
}
