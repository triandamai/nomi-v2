// The live connection behind a chat on screen (ChatThread): one socket to the chat's feed,
// reopened with backoff when it drops. What the events mean stays with the caller.

/** What the link needs from a socket: a WebSocket in the browser, a stand-in in tests. */
export interface LinkSocket {
	addEventListener(type: 'open', listener: () => void): void;
	addEventListener(type: 'message', listener: (event: { data: unknown }) => void): void;
	addEventListener(type: 'close', listener: (event: { code: number }) => void): void;
	close(): void;
}

export interface ChatLinkEvents {
	/** The socket opened; `resumed` when it came back after a drop rather than on the first open. */
	opened(resumed: boolean): void;
	message(data: unknown): void;
	/** The server ended it for good (signed out, or no access to the chat): no retry. */
	lost(): void;
}

export interface ChatLink {
	/** Connects to this chat (closing any other one), or to nothing with `null`. */
	follow(sessionId: string | null): void;
	close(): void;
}

const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
const INITIAL_RETRY_DELAY_MS = 1000;
const MAX_RETRY_DELAY_MS = 30000;

export function createChatLink(
	events: ChatLinkEvents,
	openSocket: (sessionId: string) => LinkSocket = (id) => new WebSocket(`/chat/${id}/ws`) as LinkSocket,
): ChatLink {
	let disconnect: (() => void) | null = null;

	function connectTo(id: string): () => void {
		let socket: LinkSocket | undefined;
		let retryDelay = INITIAL_RETRY_DELAY_MS;
		let retryTimeout: ReturnType<typeof setTimeout> | undefined;
		let intentionallyClosed = false;
		let hasConnectedBefore = false;

		function connect() {
			socket = openSocket(id);
			socket.addEventListener('open', () => {
				retryDelay = INITIAL_RETRY_DELAY_MS;
				events.opened(hasConnectedBefore);
				hasConnectedBefore = true;
			});
			socket.addEventListener('message', (event) => events.message(event.data));
			socket.addEventListener('close', (event) => {
				if (intentionallyClosed) return;
				if (TERMINAL_CLOSE_CODES.has(event.code)) {
					events.lost();
					return;
				}
				const jitter = Math.random() * 250;
				retryTimeout = setTimeout(connect, retryDelay + jitter);
				retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
			});
		}

		connect();
		return () => {
			intentionallyClosed = true;
			clearTimeout(retryTimeout);
			socket?.close();
		};
	}

	function follow(sessionId: string | null) {
		disconnect?.();
		disconnect = sessionId === null ? null : connectTo(sessionId);
	}

	return { follow, close: () => follow(null) };
}
