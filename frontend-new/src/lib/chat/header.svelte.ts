import { getContext, setContext } from 'svelte';

// What a chat page tells the app shell around it: its name, pin and place as they change after
// the page loaded (renamed here or live, pinned, moved), whether its name is being edited, what
// the shell's top bar and crew panel do with them, and the chats on their way out (deleted, with
// a few seconds to undo), which the sidebar leaves out. The layout owns one; the chat page
// fills it in. Changes are kept per chat, so the layout falls back to the page's own data on the
// server render and for another chat.

export type ChatAction = 'rename' | 'pin' | 'move' | 'export' | 'share' | 'delete';

export type ChatHandlers = {
	pick(action: ChatAction): void;
	rename(name: string): void;
	share(): void;
};

export type CrewInChat = { replied: string[]; working: string | null };

export class ChatHeader {
	/** The chat the changes below belong to. */
	sessionId = $state<string | null>(null);
	title = $state<string | null>(null);
	meta = $state<string | null>(null);
	pinned = $state<boolean | null>(null);
	renaming = $state(false);
	crew = $state<CrewInChat>({ replied: [], working: null });
	/** Chats deleted here, still undoable: not listed anywhere until they come back. */
	leaving = $state<string[]>([]);
	/** The sign-in ran out as this was being sent: it waits on this device (Session Expired). */
	signedOutDraft = $state<string | null>(null);
	handlers: ChatHandlers | null = null;

	/** A chat page takes it over: the changes start from nothing for that chat. */
	attach(sessionId: string, handlers: ChatHandlers) {
		if (this.sessionId !== sessionId) {
			this.sessionId = sessionId;
			this.title = null;
			this.meta = null;
			this.pinned = null;
			this.renaming = false;
			this.crew = { replied: [], working: null };
		}
		this.handlers = handlers;
	}

	detach(sessionId: string) {
		if (this.sessionId !== sessionId) return;
		this.sessionId = null;
		this.handlers = null;
		this.renaming = false;
		this.crew = { replied: [], working: null };
	}

	/** What to show for `sessionId`: the change made here, else what the page loaded. */
	shown<T>(sessionId: string, field: 'title' | 'meta' | 'pinned', loaded: T): T {
		if (this.sessionId !== sessionId || this[field] === null) return loaded;
		return this[field] as T;
	}
}

const KEY = Symbol('chat-header');

export const setChatHeader = (header: ChatHeader) => setContext(KEY, header);

export const chatHeader = (): ChatHeader => getContext<ChatHeader | undefined>(KEY) ?? new ChatHeader();
