import { describe, expect, it } from 'vitest';
import { applyChatEvent, idleTurn, isWorking, retryInfo, turnSent, turnStopped, type LiveTurn } from './turn';
import type { ChatEvent } from './events';

const T = 'turn-1';
const delta = (event: unknown, turn = T): ChatEvent => ({ kind: 'Delta', turn_job_id: turn, event });
const play = (events: [ChatEvent, number][], from: LiveTurn = turnSent(idleTurn(), 0)) => events.reduce((state, [event, at]) => applyChatEvent(state, event, at), from);

describe('the live turn', () => {
	it('works from the send until the turn completes, and times itself', () => {
		const sent = turnSent(idleTurn(), 1000);
		expect(isWorking(sent)).toBe(true);
		expect(sent.startedAt).toBe(1000);
		const streaming = play([[delta({ TextDelta: { index: 0, text: 'Food is ' } }), 1500]], sent);
		expect(streaming.openTurns).toEqual([T]);
		expect(streaming.reply.text).toBe('Food is ');
		const done = play([[{ kind: 'TurnCompleted', turn_job_id: T, message_id: 'm1' }, 2000]], streaming);
		expect(isWorking(done)).toBe(false);
		expect(done.reply.text).toBe('');
	});

	it('streams the thinking, then says how long it took once the text starts', () => {
		const state = play([
			[delta({ ContentBlockStart: { index: 0, block: 'Thinking' } }), 1000],
			[delta({ ThinkingDelta: { index: 0, text: 'Looking at October.' } }), 1000],
			[delta({ ThinkingDelta: { index: 0, text: ' Food is up.' } }), 4000],
			[delta({ TextDelta: { index: 1, text: 'Food' } }), 9000],
		]);
		expect(state.reply.thinking).toBe('Looking at October. Food is up.');
		expect(state.thoughtFor).toBe(8000);
	});

	it('knows who is answering and what they are doing', () => {
		const state = play([
			[{ kind: 'AgentSessionStarted', agent_session_id: 'a', agent_type: 'money', agent_display_name: 'Dana' }, 10],
			[{ kind: 'AgentPhaseChanged', agent_session_id: 'a', phase: 'calling_tool', detail: 'dana.records', subject: 'October', agent_display_name: 'Dana' }, 20],
		]);
		expect(state.agent).toBe('Dana');
		expect(state.phase).toEqual({ phase: 'calling_tool', detail: 'dana.records', subject: 'October' });
		const waiting = applyChatEvent(state, { kind: 'AgentPhaseChanged', agent_session_id: 'a', phase: 'waiting', detail: null }, 30);
		expect(waiting.phase).toBeNull();
	});

	it('keeps what was written when the stream is cut off, with the reason the server gave', () => {
		const state = play([
			[delta({ TextDelta: { index: 0, text: 'Most of the difference is' } }), 1000],
			[{ kind: 'TurnFailed', turn_job_id: T, error: 'DeepSeek is having trouble right now. Try again in a moment.' }, 19000],
		]);
		expect(isWorking(state)).toBe(false);
		expect(state.reply.text).toBe('Most of the difference is');
		expect(state.end).toEqual({ status: 'interrupted', elapsedMs: 19000, reason: 'DeepSeek is having trouble right now. Try again in a moment.' });
	});

	it('lets the failure notice speak when nothing was written yet', () => {
		const state = play([[{ kind: 'TurnFailed', turn_job_id: T, error: 'Something went wrong talking to the AI provider.' }, 500]]);
		expect(state.end).toBeNull();
		expect(state.reply.text).toBe('');
		expect(isWorking(state)).toBe(false);
	});

	it('hands over to the real message once it lands', () => {
		const state = play([
			[delta({ TextDelta: { index: 0, text: 'Hi' } }), 10],
			[{ kind: 'MessageCreated', message_id: 'm2' }, 20],
		]);
		expect(state.reply.text).toBe('');
	});

	it('stops: what was written stays, marked stopped, and late deltas are ignored', () => {
		const streaming = play([[delta({ TextDelta: { index: 0, text: 'Food is' } }), 1000]]);
		const stopped = turnStopped(streaming, 3000);
		expect(isWorking(stopped)).toBe(false);
		expect(stopped.end).toEqual({ status: 'stopped', elapsedMs: 3000, reason: '' });
		const late = play([[delta({ TextDelta: { index: 0, text: ' more' } }), 3100]], stopped);
		expect(late.reply.text).toBe('Food is');
		expect(isWorking(late)).toBe(false);
	});

	it('a stop with nothing written leaves nothing behind', () => {
		expect(turnStopped(turnSent(idleTurn(), 0), 10)).toMatchObject({ end: null, reply: { text: '' } });
	});

	it('a new send starts clean', () => {
		const stopped = turnStopped(play([[delta({ TextDelta: { index: 0, text: 'x' } }), 1]]), 2);
		expect(turnSent(stopped, 5)).toMatchObject({ end: null, stopRequested: false, reply: { text: '' }, startedAt: 5 });
	});

	it('settles when the crew is done with the chat', () => {
		const state = play([[{ kind: 'ChatSettled', outcome: 'done', waiting: null, agent: 'Nomi' }, 10]]);
		expect(isWorking(state)).toBe(false);
	});

	it('follows other turns (a hand-off, a scheduled run) to their own end', () => {
		const state = play(
			[
				[delta({ TextDelta: { index: 0, text: 'a' } }, 'one'), 1],
				[delta({ TextDelta: { index: 0, text: 'b' } }, 'two'), 2],
				[{ kind: 'TurnCompleted', turn_job_id: 'one', message_id: 'm' }, 3],
			],
			idleTurn(),
		);
		expect(state.openTurns).toEqual(['two']);
		expect(isWorking(state)).toBe(true);
	});
});

describe('retryInfo', () => {
	it('counts down to the next attempt from the phase detail', () => {
		const detail = JSON.stringify({ attempt: 2, max: 10, retry_at: '2026-10-21T08:14:24Z' });
		expect(retryInfo(detail, Date.parse('2026-10-21T08:14:00Z'))).toEqual({ attempt: 2, max: 10, seconds: 24 });
		expect(retryInfo(detail, Date.parse('2026-10-21T08:15:00Z'))).toEqual({ attempt: 2, max: 10, seconds: 0 });
	});

	it('reads an odd detail as now', () => {
		expect(retryInfo('nope', 0)).toEqual({ attempt: 2, max: 10, seconds: 0 });
		expect(retryInfo(null, 0)).toEqual({ attempt: 2, max: 10, seconds: 0 });
	});
});
