import { describe, expect, it } from 'vitest';
import { composeMessage, formatBytes, maxBytesFor, refusal, splitAttachments, voiceNoteName } from './attachments';

const ID = '0b6a1c5e-6f9e-4a8e-9d3e-2f1d8c7b6a50';
const tag = (name: string, kind: string, mime: string, size: number) =>
	`<attachment id="${ID}" name="${name}" kind="${kind}" mime="${mime}" size="${size}"/>`;

describe('attachments', () => {
	it('puts each uploaded file’s reference after the typed text and reads them back', () => {
		const message = composeMessage('Log these please', [tag('receipt.jpg', 'image', 'image/jpeg', 2048)]);
		expect(message.startsWith('Log these please\n\n<attachment id=')).toBe(true);
		expect(splitAttachments(message)).toEqual({
			text: 'Log these please',
			files: [{ id: ID, name: 'receipt.jpg', kind: 'image', mime: 'image/jpeg', size: 2048 }],
		});
	});

	it('sends files with no text, and text with no files', () => {
		expect(splitAttachments(composeMessage('', [tag('a.pdf', 'pdf', 'application/pdf', 9)])).text).toBe('');
		expect(splitAttachments('just words')).toEqual({ text: 'just words', files: [] });
	});

	it('still reads files older messages carried inline, in order', () => {
		const old = 'hi\n\n<attachment name="old.txt">\nbody\n</attachment>';
		expect(splitAttachments(old).files).toEqual([{ name: 'old.txt', text: 'body', kind: 'file' }]);
		const voice = '<attachment name="Voice note (0:12)" kind="voice">\nbook flights\n</attachment>';
		expect(splitAttachments(voice).files[0].kind).toBe('voice');
	});

	it('allows 10 MB a file, 50 MB a video and 100 MB a message', () => {
		const MB = 1024 * 1024;
		expect(maxBytesFor('clip.mp4', '')).toBe(50 * MB);
		expect(maxBytesFor('notes.pdf', 'application/pdf')).toBe(10 * MB);
		expect(refusal({ name: 'scan.pdf', size: 9 * MB, type: 'application/pdf' }, [])).toBeNull();
		expect(refusal({ name: 'scan.pdf', size: 11 * MB, type: 'application/pdf' }, [])).toContain('10 MB');
		expect(refusal({ name: 'trip.mov', size: 45 * MB, type: 'video/quicktime' }, [])).toBeNull();
		expect(refusal({ name: 'trip.mov', size: 60 * MB, type: 'video/quicktime' }, [])).toContain('50 MB');
		const full = Array.from({ length: 2 }, () => ({ size: 48 * MB }));
		expect(refusal({ name: 'more.mp4', size: 5 * MB, type: 'video/mp4' }, full)).toContain('100 MB');
		expect(refusal({ name: 'x.txt', size: 1, type: 'text/plain' }, Array.from({ length: 10 }, () => ({ size: 1 })))).toContain('10');
		expect(refusal({ name: 'empty.txt', size: 0, type: 'text/plain' }, [])).not.toBeNull();
	});

	it('formats sizes in bytes, KB and MB', () => {
		expect(formatBytes(512)).toBe('512 B');
		expect(formatBytes(2048)).toBe('2.0 KB');
		expect(formatBytes(3.5 * 1024 * 1024)).toBe('3.5 MB');
		expect(formatBytes(50 * 1024 * 1024)).toBe('50 MB');
	});

	it('names voice notes by length', () => {
		expect(voiceNoteName(42)).toBe('Voice note (0:42)');
		expect(voiceNoteName(125)).toBe('Voice note (2:05)');
	});
});
