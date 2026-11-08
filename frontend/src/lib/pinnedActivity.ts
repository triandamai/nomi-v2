// The activity card pinned above the composer while the crew works (ChatThread). It stays put,
// whatever else lands in the thread, until the turn settles: done (then it waits for the person's
// next message, or goes after DONE_LINGER_MS), failed, or waiting on the person (a question, an
// approval, a connection), which stay until the person answers or sends something. The server
// says how the chat settled (`ChatSettled`, backend: nomi_turn::activity); the thread's own
// messages say where the thing waiting is, and stand in when that event hasn't come.

import type { Waiting } from '$lib/chatActivity';
import type { RenderedMessage } from '$lib/types';

/** How long a finished card stays when nobody writes anything. */
export const DONE_LINGER_MS = 5_000;

/** Phases worth a line in the card's steps; thinking and writing only show as the headline. */
const STEP_PHASES = new Set(['calling_tool', 'running_command']);

/** A phase as the server sent it; `subject` is what a tool works on (a file, a search). */
export type PinnedPhase = { phase: string; detail: string | null; subject?: string | null };

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
	/** When the turn started (ms), for "started 08:13". */
	startedAt: number;
}

export const startPinned = (at: number = Date.now()): PinnedActivity => ({ status: 'working', steps: [], elapsed: null, settledAt: null, waiting: null, who: null, startedAt: at });

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
	if (last && last.phase === phase.phase && last.detail === phase.detail && (last.subject ?? null) === (phase.subject ?? null)) return card;
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

/** Where the crew's to-do stands this turn: how many are done, how many in all, and the one on
 * now (the first in progress, else the first not done), with its place in the list. */
export type TodoProgress = { done: number; total: number; current: string | null; position: number };

/** The newest to-do list posted since the person's last message, as progress. */
export function todoProgress(messages: RenderedMessage[]): TodoProgress | null {
	const lastUser = messages.findLastIndex((message) => message.sender === 'user');
	for (let at = messages.length - 1; at > lastUser; at--) {
		if (messages[at].deleted) continue;
		const list = messages[at].content_blocks?.findLast((block) => block.kind === 'todo_list');
		if (list?.kind !== 'todo_list' || list.items.length === 0) continue;
		const items = list.items;
		const done = items.filter((item) => item.status === 'done').length;
		let on = items.findIndex((item) => item.status === 'in_progress');
		if (on < 0) on = items.findIndex((item) => item.status !== 'done');
		return { done, total: items.length, current: on >= 0 ? items[on].text : null, position: on >= 0 ? on + 1 : items.length };
	}
	return null;
}

const MINUTE_MS = 60_000;

/** About how many minutes are left, going at the pace of the items done so far; null until one
 * is done (there's no pace yet) or once all are. */
export function minutesLeft(progress: TodoProgress | null, workedMs: number): number | null {
	if (!progress || progress.done < 1 || progress.done >= progress.total) return null;
	const perItem = workedMs / progress.done;
	return Math.max(1, Math.ceil((perItem * (progress.total - progress.done)) / MINUTE_MS));
}
