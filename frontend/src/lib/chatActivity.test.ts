import { describe, expect, it } from 'vitest';
import { toastFor } from './chatActivity';

describe('a chat the person left', () => {
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
