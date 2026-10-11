import { describe, expect, it } from 'vitest';
import { composeAnswers } from './chatSend';

describe('composeAnswers', () => {
	it('pairs each answered question with the picks, then the note', () => {
		const questions = [{ question: 'Which look?' }, { question: 'Which pages?' }, { question: 'Sign-in?' }];
		expect(composeAnswers(questions, [['Editorial'], ['Menu', 'Location'], []], ' Open 7 to 3 ')).toBe(
			'Which look? Editorial\nWhich pages? Menu, Location\nOpen 7 to 3',
		);
	});

	it('is empty when nothing was picked or written', () => {
		expect(composeAnswers([{ question: 'Q?' }], [[]], '  ')).toBe('');
	});
});
