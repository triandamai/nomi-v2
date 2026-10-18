import { toastFor, type ChatOutcome, type ChatToast, type Waiting } from '$lib/chatActivity';
import { showSnackbar } from '$lib/snackbars.svelte';

// Chats the person left while the crew was still working on them (they went to another chat,
// another page, or another tab). Each keeps listening on the chat's live connection until the
// work is over, then says how it went in a snackbar with a way back (see the app layout).
//
// A chat counts as over when every turn streaming into it has ended and no crew member still has
// work queued or running for it (its delegations). Coming back to the chat stops watching it: the
// chat shows the rest itself. The list survives a reload of the tab (sessionStorage).

const NIL_UUID = '00000000-0000-0000-0000-000000000000';
/** After the last turn ends, how long to wait for a hand-off to start before calling it over. */
const SETTLE_MS = 2500;
/** While a crew member still has work queued, how often to look again (besides live events). */
const RECHECK_MS = 20_000;
/** A chat left before its turn started streaming: when to look whether it already answered. */
const FIRST_LOOK_MS = 15_000;
/** A chat watched this long without ending is let go silently. */
const MAX_WATCH_MS = 45 * 60_000;
const STORAGE_KEY = 'nomi.background-chats';
const CHANNEL = 'nomi-chat-activity';

export interface WatchedChat {
	sessionId: string;
	/** Where the chat lives: /chat/… or a project's /projects/session/…. */
	href: string;
	title: string;
	/** Turns still streaming into it when it was left. */
	openTurns?: string[];
	/** When the work being watched began (ms); only messages from then on count. */
	since?: number;
}

interface Watch extends Required<WatchedChat> {
	socket: WebSocket | null;
	/** A turn has been seen (or was open when the chat was left), so the work has started. */
	started: boolean;
	/** A turn was seen ending, so the crew has answered (or failed). */
	ended: boolean;
	failed: boolean;
	settle: ReturnType<typeof setTimeout> | undefined;
	giveUp: ReturnType<typeof setTimeout>;
	retries: number;
}

const watches = new Map<string, Watch>();

function persist() {
	try {
		const list = [...watches.values()].map(({ sessionId, href, title, since }) => ({ sessionId, href, title, since }));
		sessionStorage.setItem(STORAGE_KEY, JSON.stringify(list));
	} catch {
		// Storage off: watching just doesn't survive a reload.
	}
}

let channel: BroadcastChannel | null = null;
function broadcast(): BroadcastChannel | null {
	if (!channel && typeof BroadcastChannel !== 'undefined') channel = new BroadcastChannel(CHANNEL);
	return channel;
}

/** Shows a snackbar here, or, when this tab is hidden, in whichever tab the person is looking at. */
function announce(toast: ChatToast) {
	if (typeof document !== 'undefined' && document.hidden) {
		broadcast()?.postMessage(toast);
		return;
	}
	if (location.pathname === toast.href) return; // already looking at it
	showSnackbar(toast);
}

/** Snackbars announced by other tabs (a project open in its own tab finishing). Returns the stop. */
export function listenToOtherTabs(): () => void {
	const bc = broadcast();
	if (!bc) return () => {};
	const onMessage = (event: MessageEvent<ChatToast>) => {
		if (document.hidden || location.pathname === event.data.href) return;
		showSnackbar(event.data);
	};
	bc.addEventListener('message', onMessage);
	return () => bc.removeEventListener('message', onMessage);
}

/** Starts watching a chat the person is leaving while the crew works on it. */
export function watchChat(chat: WatchedChat) {
	if (typeof WebSocket === 'undefined') return;
	stop(chat.sessionId);
	const openTurns = chat.openTurns ?? [];
	const watch: Watch = {
		sessionId: chat.sessionId,
		href: chat.href,
		title: chat.title,
		openTurns,
		since: chat.since ?? Date.now(),
		socket: null,
		started: openTurns.length > 0,
		ended: false,
		failed: false,
		settle: undefined,
		giveUp: setTimeout(() => stop(chat.sessionId), MAX_WATCH_MS),
		retries: 0,
	};
	watches.set(chat.sessionId, watch);
	persist();
	connect(watch);
}

/** Stops watching a chat (the person came back to it). */
export function unwatchChat(sessionId: string) {
	stop(sessionId);
}

/** Picks the watched chats back up after a reload of the tab. */
export function resumeWatches() {
	let list: WatchedChat[] = [];
	try {
		list = JSON.parse(sessionStorage.getItem(STORAGE_KEY) ?? '[]');
	} catch {
		return;
	}
	for (const chat of list) {
		if (watches.has(chat.sessionId) || location.pathname === chat.href) continue;
		watchChat(chat);
		// What happened during the reload came with no events: the connection looks once it opens.
		const watch = watches.get(chat.sessionId);
		if (watch) watch.started = true;
	}
}

function stop(sessionId: string) {
	const watch = watches.get(sessionId);
	if (!watch) return;
	watches.delete(sessionId);
	clearTimeout(watch.settle);
	clearTimeout(watch.giveUp);
	const socket = watch.socket;
	watch.socket = null;
	socket?.close();
	persist();
}

function connect(watch: Watch) {
	const socket = new WebSocket(`/chat/${watch.sessionId}/ws`);
	watch.socket = socket;
	socket.addEventListener('open', () => {
		// Anything that ended before this connected came with no event: look once (a turn not
		// streaming yet gets longer to start).
		scheduleSettle(watch, watch.started ? SETTLE_MS : FIRST_LOOK_MS);
		watch.retries = 0;
	});
	socket.addEventListener('message', (event) => {
		let envelope: { kind: string; turn_job_id?: string; error?: string };
		try {
			envelope = JSON.parse(event.data);
		} catch {
			return;
		}
		const turn = envelope.turn_job_id ?? NIL_UUID;
		if (envelope.kind === 'Delta') {
			watch.started = true;
			if (!watch.openTurns.includes(turn)) watch.openTurns = [...watch.openTurns, turn];
			clearTimeout(watch.settle);
		} else if (envelope.kind === 'TurnCompleted' || envelope.kind === 'TurnFailed') {
			watch.started = true;
			watch.ended = true;
			if (envelope.kind === 'TurnFailed') watch.failed = true;
			watch.openTurns = watch.openTurns.filter((id) => id !== turn);
			if (watch.openTurns.length === 0) scheduleSettle(watch, SETTLE_MS);
		} else if (envelope.kind === 'AgentDelegationUpdated' && watch.openTurns.length === 0) {
			scheduleSettle(watch, SETTLE_MS);
		}
	});
	socket.addEventListener('close', () => {
		if (watch.socket !== socket || watches.get(watch.sessionId) !== watch) return;
		watch.socket = null;
		if (watch.retries >= 5) return stop(watch.sessionId);
		watch.retries += 1;
		setTimeout(() => watches.get(watch.sessionId) === watch && connect(watch), 2000 * watch.retries);
	});
}

function scheduleSettle(watch: Watch, delay: number) {
	clearTimeout(watch.settle);
	watch.settle = setTimeout(() => void settle(watch), delay);
}

/** Once nothing streams: finished, or is a crew member still at it? */
async function settle(watch: Watch) {
	if (watches.get(watch.sessionId) !== watch || watch.openTurns.length > 0) return;
	let state: { busy: boolean; replied: boolean; waiting: Waiting | null; agent: string | null };
	try {
		const response = await fetch(`/chat/${watch.sessionId}/activity?since=${encodeURIComponent(new Date(watch.since - 5000).toISOString())}`);
		if (!response.ok) return stop(watch.sessionId);
		state = await response.json();
	} catch {
		return scheduleSettle(watch, RECHECK_MS);
	}
	if (watches.get(watch.sessionId) !== watch || watch.openTurns.length > 0) return;
	// Still at it: a crew member has work queued, or the turn hasn't answered yet (and no end was
	// seen, which would have been the answer, or a failure that posts its own notice).
	if (state.busy || (!state.replied && !watch.ended && !state.waiting)) return scheduleSettle(watch, RECHECK_MS);
	const outcome: ChatOutcome = state.waiting ? { kind: 'waiting', on: state.waiting } : watch.failed ? { kind: 'failed' } : { kind: 'done' };
	stop(watch.sessionId);
	announce(toastFor(outcome, { title: watch.title, href: watch.href }, state.agent));
}
