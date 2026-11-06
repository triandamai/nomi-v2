// The activity card pinned above the composer while the crew works (ChatThread). It stays put,
// whatever else lands in the thread, until the turn settles: done (then it waits for the person's
// next message, or goes after DONE_LINGER_MS), failed, or waiting on the person (a question, an
// approval, a connection), which stay until the person answers or sends something. The server
// says how the chat settled (`ChatSettled`, backend: nomi_turn::activity); the thread's own
// messages say where the thing waiting is, and stand in when that event hasn't come.

import type { Waiting } from '$lib/chatActivity';
import type { RenderedMessage } from '$lib/types';

/** How long a finished card stays when nobody writes anything. */
export const DONE_LINGER_MS = 60_000;

/** Phases worth a line in the card's steps; thinking and writing only show as the headline. */
const STEP_PHASES = new Set(['calling_tool', 'running_command']);

export type PinnedPhase = { phase: string; detail: string | null };

export interface PinnedActivity {
	status: 'working' | 'done' | 'failed';
	/** The steps taken this turn, oldest first; the last is running while the turn is. */
	steps: PinnedPhase[];
	/** The time the turn took, once it settled ("01:12"). */
	elapsed: string | null;
	/** When it settled (ms), for DONE_LINGER_MS. */
	settledAt: number | null;
	/** What the server said the crew is waiting on the person for, until the thread moves on. */
	waiting: Waiting | null;
	/** Who's on it, as the server named them (the agent's phase, or who answered last). */
	who: string | null;
}

export const startPinned = (): PinnedActivity => ({ status: 'working', steps: [], elapsed: null, settledAt: null, waiting: null, who: null });

const WAITING: readonly string[] = ['question', 'approval', 'connect'] satisfies Waiting[];

/** The server's word on how the chat settled (`ChatSettled`). A card that wasn't there (the page
 * opened mid-turn) starts from it. */
export function withSettled(
	card: PinnedActivity | null,
	event: { outcome: string; waiting: string | null; agent: string | null },
	elapsed: string | null,
	at: number,
): PinnedActivity {
	const base = card ?? startPinned();
	const waiting = event.waiting && WAITING.includes(event.waiting) ? (event.waiting as Waiting) : null;
	const status = event.outcome === 'failed' ? 'failed' : 'done';
	return { ...base, status, waiting, who: event.agent ?? base.who, elapsed: base.elapsed ?? elapsed, settledAt: base.settledAt ?? at };
}

/** A phase change while working: a tool or command adds a step unless it repeats the last one. */
export function withPhase(card: PinnedActivity, phase: PinnedPhase): PinnedActivity {
	if (card.status !== 'working' || !STEP_PHASES.has(phase.phase)) return card;
	const last = card.steps.at(-1);
	if (last && last.phase === phase.phase && last.detail === phase.detail) return card;
	return { ...card, steps: [...card.steps, phase] };
}

export function settled(card: PinnedActivity, status: 'done' | 'failed', elapsed: string, at: number): PinnedActivity {
	return { ...card, status, elapsed, settledAt: at };
}

/** A finished card has outstayed DONE_LINGER_MS. Failed cards stay until handled. */
export const lingeredOut = (card: PinnedActivity, now: number): boolean =>
	card.status === 'done' && card.settledAt !== null && now - card.settledAt >= DONE_LINGER_MS;

/** What the crew left waiting on the person since their last message, and where: the newest
 * unanswered question, pending approval or connection card. Questions before the person's last
 * message were answered by it, so only what came after counts. */
export function waitingOn(messages: RenderedMessage[]): { on: Waiting; messageId: string } | null {
	const lastUser = messages.findLastIndex((message) => message.sender === 'user');
	for (let at = messages.length - 1; at > lastUser; at--) {
		const message = messages[at];
		if (message.deleted || message.sender !== 'assistant') continue;
		for (const block of message.content_blocks ?? []) {
			if (block.kind === 'approval_request' && block.status === 'pending') return { on: 'approval', messageId: message.id };
			if (block.kind === 'questions') return { on: 'question', messageId: message.id };
			if (block.kind === 'workspace_connect' || block.kind === 'connector_connect') return { on: 'connect', messageId: message.id };
			if (block.kind === 'view' && (block.view === 'decision' || block.view === 'form')) return { on: 'question', messageId: message.id };
		}
	}
	return null;
}

/** The id of the message saying why the turn failed, posted since the person's last message, if any. */
export function failureMessageId(messages: RenderedMessage[]): string | null {
	const lastUser = messages.findLastIndex((message) => message.sender === 'user');
	for (let at = messages.length - 1; at > lastUser; at--) {
		if (messages[at].content_blocks?.some((block) => block.kind === 'run_failed')) return messages[at].id;
	}
	return null;
}
