import { describe, expect, it } from 'vitest';
import { agentName, applyFrame, eventSentence, liveRows, parseFrame, phaseLabel, shortId, type LiveState } from './agents';
import type { AgentEvent, AgentsResponse } from './api';

const snapshot: AgentsResponse = {
	users: [
		{
			user_id: 'u1',
			label: 'ana@nomi.test',
			agents: [
				{
					agent_session_id: 'as-1',
					session_id: 's-1111111111',
					agent_type: 'money',
					agent_display_name: 'Money',
					channel: 'web',
					started_at: '2026-11-10T08:00:00Z',
					last_activity_at: '2026-11-10T08:01:00Z',
					current_phase: 'thinking',
					current_phase_detail: null,
				},
			],
		},
	],
};
const NOW = '2026-11-10T09:00:00.000Z';
const start = (): LiveState => ({ rows: liveRows(snapshot), feed: [] });

describe('the crew at work', () => {
	it('lists everyone’s running agents with the person they work for', () => {
		expect(liveRows(snapshot)).toEqual([expect.objectContaining({ agent_session_id: 'as-1', person: 'ana@nomi.test', agent_type: 'money', current_phase: 'thinking' })]);
	});

	it('names built-in agents as the crew does, whichever name the backend sent, and custom ones as they are called', () => {
		expect(agentName('money', 'Money')).toBe('Dana');
		expect(agentName('coding', 'Koda')).toBe('Koda');
		expect(agentName('8f2c-dyn', 'Pajak')).toBe('Pajak');
		expect(agentName(null, null)).toBe('An agent');
	});

	it('adds an agent that starts, once, even when the snapshot already had it', () => {
		const started = { kind: 'AgentSessionStarted', agent_session_id: 'as-2', session_id: 's-2', agent_type: 'coding', agent_display_name: 'Koda', channel: 'web', sender_label: 'budi@nomi.test' } as const;
		let next = applyFrame(start(), started, NOW);
		expect(next.rows.map((row) => row.agent_session_id)).toEqual(['as-1', 'as-2']);
		expect(next.rows[1]).toMatchObject({ person: 'budi@nomi.test', current_phase: 'waiting', started_at: NOW });
		expect(next.feed[0]).toMatchObject({ event_type: 'AgentSpawned', agent_type: 'coding', session_id: 's-2' });
		next = applyFrame(next, { ...started, agent_session_id: 'as-1' }, NOW);
		expect(next.rows.map((row) => row.agent_session_id)).toEqual(['as-1', 'as-2']);
	});

	it('moves an agent to its new step, and says in the feed when it uses a tool or writes the reply', () => {
		let next = applyFrame(start(), { kind: 'AgentPhaseChanged', agent_session_id: 'as-1', session_id: 's-1', phase: 'calling_tool', detail: 'list_transactions' }, NOW);
		expect(next.rows[0]).toMatchObject({ current_phase: 'calling_tool', current_phase_detail: 'list_transactions', last_activity_at: NOW });
		expect(next.feed[0]).toMatchObject({ event_type: 'ToolCalled', tool_name: 'list_transactions', agent_type: 'money' });
		next = applyFrame(next, { kind: 'AgentPhaseChanged', agent_session_id: 'as-1', session_id: 's-1', phase: 'writing_reply', detail: null }, NOW);
		expect(next.feed[0]).toMatchObject({ event_type: 'AgentFinalizing' });
		next = applyFrame(next, { kind: 'AgentPhaseChanged', agent_session_id: 'as-1', session_id: 's-1', phase: 'thinking', detail: null }, NOW);
		expect(next.feed).toHaveLength(2);
	});

	it('takes an agent off the list when it ends, saying how', () => {
		const next = applyFrame(start(), { kind: 'AgentSessionEnded', agent_session_id: 'as-1', session_id: 's-1', reason: 'expired' }, NOW);
		expect(next.rows).toEqual([]);
		expect(next.feed[0]).toMatchObject({ event_type: 'AgentExpired', agent_type: 'money' });
		expect(applyFrame(start(), { kind: 'AgentSessionEnded', agent_session_id: 'as-1', session_id: 's-1', reason: 'cancelled' }, NOW).feed[0].event_type).toBe('AgentCancelled');
		expect(applyFrame(start(), { kind: 'AgentSessionEnded', agent_session_id: 'as-1', session_id: 's-1', reason: 'completed' }, NOW).feed[0].event_type).toBe('AgentCompleted');
	});

	it('keeps the feed to the latest 200', () => {
		let next: LiveState = { rows: [], feed: [] };
		for (let index = 0; index < 205; index++) next = applyFrame(next, { kind: 'AgentPhaseChanged', agent_session_id: 'x', session_id: 's', phase: 'writing_reply', detail: null }, NOW);
		expect(next.feed).toHaveLength(200);
	});

	it('reads only frames it knows', () => {
		expect(parseFrame('{"kind":"AgentSessionEnded","agent_session_id":"a","session_id":"s","reason":"completed"}')).toMatchObject({ kind: 'AgentSessionEnded' });
		expect(parseFrame('{"kind":"Delta"}')).toBeNull();
		expect(parseFrame('not json')).toBeNull();
		expect(parseFrame(42)).toBeNull();
	});
});

describe('what the crew is doing, in words', () => {
	it('says each step plainly', () => {
		expect(phaseLabel('waiting', null)).toBe('waiting');
		expect(phaseLabel('thinking', null)).toBe('thinking');
		expect(phaseLabel('calling_tool', 'read_file')).toBe('using read_file');
		expect(phaseLabel('calling_tool', null)).toBe('using a tool');
		expect(phaseLabel('writing_reply', null)).toBe('writing the reply');
		expect(phaseLabel('retrying', '{"attempt":2,"max":5,"retry_at":"x"}')).toBe('retrying, 2 of 5');
		expect(phaseLabel('retrying', 'garbled')).toBe('retrying');
		expect(phaseLabel('running_command', '{"command":"npm test","started_at":"x"}')).toBe('running a command');
		expect(phaseLabel('something_new', null)).toBe('something new');
	});

	it('tells each event as a sentence, naming the chat by its short id', () => {
		const event = (patch: Partial<AgentEvent>): AgentEvent => ({
			id: 'e',
			session_id: 'a1b2c3d4e5f6',
			agent_session_id: 'as',
			agent_type: 'money',
			agent_display_name: 'Money',
			event_type: 'AgentSpawned',
			created_at: NOW,
			tool_name: null,
			is_error: null,
			...patch,
		});
		expect(shortId('a1b2c3d4e5f6')).toBe('a1b2c3d4');
		expect(eventSentence(event({}))).toBe('Dana picked up chat a1b2c3d4');
		expect(eventSentence(event({ event_type: 'ToolCalled', tool_name: 'log_transaction', is_error: true }))).toBe('Dana used log_transaction in chat a1b2c3d4, and it failed');
		expect(eventSentence(event({ event_type: 'AgentReplied', session_id: null }))).toBe('Dana replied');
		expect(eventSentence(event({ event_type: 'Mystery', agent_type: null, agent_display_name: null }))).toBe('An agent: Mystery in chat a1b2c3d4');
	});
});
