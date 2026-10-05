import { describe, expect, it } from 'vitest';
import { composeMessage, isAudioFile, isTextFile, splitAttachments, voiceNoteName } from './attachments';

describe('attachments', () => {
	it('round-trips typed text and files', () => {
		const files = [
			{ name: 'notes.md', text: '# Trip\n- flights', kind: 'file' as const },
			{ name: 'Voice note (0:12)', text: 'remind me to book flights friday', kind: 'voice' as const },
		];
		const message = composeMessage('Can you check these?', files);
		expect(message.startsWith('Can you check these?')).toBe(true);
		expect(splitAttachments(message)).toEqual({ text: 'Can you check these?', files });
	});

	it('works with files and no text, and with text and no files', () => {
		expect(splitAttachments(composeMessage('', [{ name: 'a.txt', text: 'x' }]))).toEqual({ text: '', files: [{ name: 'a.txt', text: 'x', kind: 'file' }] });
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

describe('voice notes and older messages', () => {
	it('reads sections written before kinds existed', () => {
		expect(splitAttachments('hi\n\n<attachment name="old.txt">\nbody\n</attachment>').files).toEqual([{ name: 'old.txt', text: 'body', kind: 'file' }]);
	});
	it('names voice notes by length and spots audio files', () => {
		expect(voiceNoteName(42)).toBe('Voice note (0:42)');
		expect(voiceNoteName(125)).toBe('Voice note (2:05)');
		expect(isAudioFile('memo.m4a', '')).toBe(true);
		expect(isAudioFile('notes.md', 'text/markdown')).toBe(false);
	});
});
