import { applyStreamEvent, emptyStream, type LiveReply } from './liveReply';
import type { ChatEvent } from './events';

// The turn in progress, from the chat's live feed: from the moment the person sends until every
// turn running in the chat has ended. It holds the reply as it streams (thinking, then text), who
// is answering and what they're doing, how long it's been going, and how it ended when it ended
// short of a message (stopped by the person, or cut off with the provider's reason). Pure: the
// page feeds it events and the clock. Logic from the old app's ChatThread (its live handling).

export type Phase = { phase: string; detail: string | null; subject: string | null };

export type LiveTurn = {
	/** Sent, and no turn has said anything yet. */
	pending: boolean;
	/** Turns streaming into this chat right now (the person's own, a hand-off, a scheduled run). */
	openTurns: string[];
	/** When the person sent, or the first turn spoke. */
	startedAt: number | null;
	reply: LiveReply;
	thinkingStartedAt: number | null;
	/** How long the thinking took, once the text started. */
	thoughtFor: number | null;
	/** Who's answering (a crew member Nomi handed it to). */
	agent: string | null;
	phase: Phase | null;
	/** The person stopped the crew: what streams after that is ignored. */
	stopRequested: boolean;
	/** The reply ended short of a message, with what was written kept. */
	end: { status: 'stopped' | 'interrupted'; elapsedMs: number; reason: string } | null;
};

export const idleTurn = (): LiveTurn => ({
	pending: false,
	openTurns: [],
	startedAt: null,
	reply: emptyStream(),
	thinkingStartedAt: null,
	thoughtFor: null,
	agent: null,
	phase: null,
	stopRequested: false,
	end: null,
});

export const isWorking = (turn: LiveTurn) => turn.pending || turn.openTurns.length > 0;

/** The person sent a message: a new turn is on its way. */
export function turnSent(turn: LiveTurn, now: number): LiveTurn {
	return { ...idleTurn(), pending: true, startedAt: now, agent: turn.openTurns.length ? turn.agent : null };
}

/** The supervisor answered a stop: nothing more is coming, and what was written stays as stopped. */
export function turnStopped(turn: LiveTurn, now: number): LiveTurn {
	const written = turn.reply.text.trim() !== '';
	return {
		...turn,
		pending: false,
		openTurns: [],
		phase: null,
		stopRequested: true,
		reply: written ? turn.reply : emptyStream(),
		end: written ? { status: 'stopped', elapsedMs: now - (turn.startedAt ?? now), reason: '' } : null,
	};
}

/** Settles when nothing is left running. */
function closeTurn(turn: LiveTurn, id: string): LiveTurn {
	const openTurns = turn.openTurns.filter((open) => open !== id);
	return { ...turn, openTurns, pending: false, ...(openTurns.length === 0 && { phase: null }) };
}

export function applyChatEvent(turn: LiveTurn, event: ChatEvent, now: number): LiveTurn {
	switch (event.kind) {
		case 'Delta': {
			if (turn.stopRequested) return turn;
			const reply = applyStreamEvent(turn.end ? emptyStream() : turn.reply, event.event);
			const thinkingStartedAt = turn.thinkingStartedAt ?? (reply.thinking && !turn.reply.thinking ? now : null);
			const textStarted = reply.text !== '' && turn.reply.text === '';
			return {
				...turn,
				pending: false,
				openTurns: turn.openTurns.includes(event.turn_job_id) ? turn.openTurns : [...turn.openTurns, event.turn_job_id],
				startedAt: turn.startedAt ?? now,
				reply,
				end: null,
				thinkingStartedAt,
				thoughtFor: turn.thoughtFor ?? (textStarted && thinkingStartedAt !== null ? now - thinkingStartedAt : null),
			};
		}
		case 'TurnCompleted':
			return { ...closeTurn(turn, event.turn_job_id), reply: turn.end ? turn.reply : emptyStream(), stopRequested: false };
		case 'TurnFailed': {
			const closed = closeTurn(turn, event.turn_job_id);
			// Cut off part-way: what was written stays, with the reason the server gave. With nothing
			// written, the failure notice it posts in the chat says it instead.
			if (turn.reply.text.trim() && !turn.stopRequested) {
				return { ...closed, end: { status: 'interrupted', elapsedMs: now - (turn.startedAt ?? now), reason: event.error } };
			}
			return { ...closed, reply: emptyStream(), stopRequested: false };
		}
		case 'MessageCreated':
			// A posted message is what was streaming: the real one takes over.
			return turn.end ? turn : { ...turn, reply: emptyStream() };
		case 'AgentSessionStarted':
			return turn.stopRequested ? turn : { ...turn, agent: event.agent_display_name };
		case 'AgentPhaseChanged':
			if (turn.stopRequested) return turn;
			return {
				...turn,
				agent: event.agent_display_name ?? turn.agent,
				phase: event.phase === 'waiting' ? null : { phase: event.phase, detail: event.detail ?? null, subject: event.subject ?? null },
			};
		case 'ChatSettled':
			return { ...turn, pending: false, openTurns: [], phase: null };
		default:
			return turn;
	}
}

/** The provider failed and the crew tries again on its own (phase `retrying`): which attempt, of
 * how many, and how long until it. An odd detail reads as now. */
export function retryInfo(detail: string | null, now: number): { attempt: number; max: number; seconds: number } {
	let attempt = 2;
	let max = 10;
	let seconds = 0;
	try {
		const parsed = JSON.parse(detail ?? '{}') as { attempt?: number; max?: number; retry_at?: string };
		attempt = parsed.attempt ?? attempt;
		max = parsed.max ?? max;
		if (parsed.retry_at) seconds = Math.max(0, Math.ceil((Date.parse(parsed.retry_at) - now) / 1000));
	} catch {
		// An older or odd detail: the generic wording still reads right.
	}
	return { attempt, max, seconds };
}
