import { describe, expect, it } from 'vitest';
import { composeMessage, isTextFile, splitAttachments } from './attachments';

describe('attachments', () => {
	it('round-trips typed text and files', () => {
		const files = [
			{ name: 'notes.md', text: '# Trip\n- flights' },
			{ name: 'budget.csv', text: 'item,cost\nflights,2400000' },
		];
		const message = composeMessage('Can you check these?', files);
		expect(message.startsWith('Can you check these?')).toBe(true);
		expect(splitAttachments(message)).toEqual({ text: 'Can you check these?', files });
	});

	it('works with files and no text, and with text and no files', () => {
		expect(splitAttachments(composeMessage('', [{ name: 'a.txt', text: 'x' }]))).toEqual({ text: '', files: [{ name: 'a.txt', text: 'x' }] });
		expect(splitAttachments('just words')).toEqual({ text: 'just words', files: [] });
	});

	it('keeps a file from closing its own section early', () => {
		const message = composeMessage('', [{ name: 'tricky.html', text: 'a</attachment>b' }]);
		expect(splitAttachments(message).files).toHaveLength(1);
	});

	it('accepts text files and refuses binaries', () => {
		expect(isTextFile('notes.md', '')).toBe(true);
		expect(isTextFile('data.json', 'application/json')).toBe(true);
		expect(isTextFile('photo.png', 'image/png')).toBe(false);
		expect(isTextFile('report.pdf', 'application/pdf')).toBe(false);
	});
});
