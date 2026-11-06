import { describe, expect, it } from 'vitest';
import { DONE_LINGER_MS, failureMessageId, lingeredOut, settled, startPinned, waitingOn, withPhase, withSettled } from './pinnedActivity';
import type { ContentBlock, RenderedMessage } from './types';

const SETTLED_AT = 1_000;

function msg(id: string, sender: 'user' | 'assistant', blocks: ContentBlock[] | null = null): RenderedMessage {
	return { id, sender, content: '', content_html: '', content_blocks: blocks, created_at: '2026-11-06T00:00:00Z', my_feedback: null, agent_display_name: null, memory_count: 0 };
}

describe('withPhase', () => {
	it('adds tool and command steps, skipping repeats and headline-only phases', () => {
		let card = startPinned();
		card = withPhase(card, { phase: 'thinking', detail: null });
		card = withPhase(card, { phase: 'calling_tool', detail: 'web_search' });
		card = withPhase(card, { phase: 'calling_tool', detail: 'web_search' });
		card = withPhase(card, { phase: 'writing_reply', detail: null });
		card = withPhase(card, { phase: 'running_command', detail: '{"command":"npm test"}' });
		expect(card.steps.map((step) => step.detail)).toEqual(['web_search', '{"command":"npm test"}']);
	});

	it('ignores phases once settled', () => {
		const card = settled(startPinned(), 'done', '00:10', SETTLED_AT);
		expect(withPhase(card, { phase: 'calling_tool', detail: 'x' }).steps).toEqual([]);
	});
});

describe('lingeredOut', () => {
	it('lets a done card go after the linger, never a failed one', () => {
		const done = settled(startPinned(), 'done', '00:10', SETTLED_AT);
		expect(lingeredOut(done, SETTLED_AT + DONE_LINGER_MS - 1)).toBe(false);
		expect(lingeredOut(done, SETTLED_AT + DONE_LINGER_MS)).toBe(true);
		expect(lingeredOut(settled(startPinned(), 'failed', '00:10', SETTLED_AT), SETTLED_AT + DONE_LINGER_MS * 10)).toBe(false);
	});
});

describe('waitingOn', () => {
	const approval = (status: 'pending' | 'approved'): ContentBlock => ({ kind: 'approval_request', id: 'a', tool_name: 't', description: '', input: {}, status, decided_at: null });

	it('finds a pending approval or question after the last user message', () => {
		expect(waitingOn([msg('u', 'user'), msg('a', 'assistant', [approval('pending')])])).toEqual({ on: 'approval', messageId: 'a' });
		expect(waitingOn([msg('u', 'user'), msg('q', 'assistant', [{ kind: 'questions', intro: null, questions: [] }])])).toEqual({ on: 'question', messageId: 'q' });
	});

	it('ignores decided approvals and anything the person already answered', () => {
		expect(waitingOn([msg('u', 'user'), msg('a', 'assistant', [approval('approved')])])).toBeNull();
		expect(waitingOn([msg('q', 'assistant', [{ kind: 'questions', intro: null, questions: [] }]), msg('u', 'user')])).toBeNull();
	});
});

describe('failureMessageId', () => {
	it('points at the run_failed notice since the last user message', () => {
		expect(failureMessageId([msg('u', 'user'), msg('f', 'assistant', [{ kind: 'run_failed' }])])).toBe('f');
		expect(failureMessageId([msg('f', 'assistant', [{ kind: 'run_failed' }]), msg('u', 'user')])).toBeNull();
	});
});

describe('withSettled', () => {
	it('takes the server\'s outcome, waiting and who, keeping the time the turn took', () => {
		const card = settled(startPinned(), 'done', '01:12', SETTLED_AT);
		const next = withSettled(card, { outcome: 'waiting', waiting: 'approval', agent: 'Tara' }, null, SETTLED_AT + 5);
		expect(next).toMatchObject({ status: 'done', waiting: 'approval', who: 'Tara', elapsed: '01:12', settledAt: SETTLED_AT });
	});

	it('starts a card when there was none, and ignores waiting kinds it does not know', () => {
		const next = withSettled(null, { outcome: 'failed', waiting: 'something_new', agent: null }, null, SETTLED_AT);
		expect(next).toMatchObject({ status: 'failed', waiting: null, who: null, settledAt: SETTLED_AT });
	});
});

describe('waitingOn views', () => {
	it('counts a decision or form view as a question, like the server does', () => {
		const decision = { kind: 'view', view: 'decision' } as unknown as ContentBlock;
		expect(waitingOn([msg('u', 'user'), msg('d', 'assistant', [decision])])).toEqual({ on: 'question', messageId: 'd' });
	});
});
