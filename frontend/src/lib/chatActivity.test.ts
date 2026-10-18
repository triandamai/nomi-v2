import { describe, expect, it } from 'vitest';
import { toastFor, waitingIn } from './chatActivity';
import type { ContentBlock, MessageItem } from './types';

const said = (sender: 'user' | 'assistant', ...blocks: ContentBlock[]): Pick<MessageItem, 'sender' | 'content_blocks'> => ({
	sender,
	content_blocks: blocks.length ? blocks : null,
});

describe('a chat the person left', () => {
	it('is waiting on them for a question, a decision, a form, an approval or an account', () => {
		expect(waitingIn([said('assistant', { kind: 'questions', intro: null, questions: [] })])).toBe('question');
		expect(waitingIn([said('assistant', { kind: 'view', view: 'decision', question: 'Which?', options: [] })])).toBe('question');
		expect(waitingIn([said('assistant', { kind: 'view', view: 'form', fields: [] })])).toBe('question');
		const approval = { kind: 'approval_request', id: 'a', tool_name: 'gmail_send', description: '', input: {}, decided_at: null } as const;
		expect(waitingIn([said('assistant', { ...approval, status: 'pending' })])).toBe('approval');
		expect(waitingIn([said('assistant', { kind: 'connector_connect', connector: 'hevy', reason: 'not_connected' })])).toBe('connect');
	});

	it('is not waiting on them for a plain reply, a shown view, a decided approval or their own words', () => {
		expect(waitingIn([said('assistant')])).toBeNull();
		expect(waitingIn([said('assistant', { kind: 'view', view: 'answer', value: 3 })])).toBeNull();
		const approval = { kind: 'approval_request', id: 'a', tool_name: 'x', description: '', input: {}, decided_at: null, status: 'approved' } as const;
		expect(waitingIn([said('assistant', approval)])).toBeNull();
		expect(waitingIn([said('user', { kind: 'questions', intro: null, questions: [] })])).toBeNull();
	});

	it('says what happened, with a way back that fits', () => {
		const chat = { title: 'Trip to Bandung', href: '/chat/1' };
		const done = toastFor({ kind: 'done' }, chat, 'Finley');
		expect(done.message).toBe('Finley finished in “Trip to Bandung”');
		expect([done.actionLabel, done.href]).toEqual(['Open', '/chat/1']);
		const question = toastFor({ kind: 'waiting', on: 'question' }, chat, null);
		expect(question.message).toBe('Nomi has a question for you in “Trip to Bandung”');
		expect(question.actionLabel).toBe('Answer');
		expect(question.timeout).toBeGreaterThan(done.timeout);
		expect(toastFor({ kind: 'waiting', on: 'approval' }, chat, 'Tara').actionLabel).toBe('Review');
		expect(toastFor({ kind: 'failed' }, { title: ' ', href: '/chat/2' }, null).message).toBe('Something went wrong in “Chat”');
	});
});
