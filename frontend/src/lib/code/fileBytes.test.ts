import { describe, expect, it } from 'vitest';
import { base64ToBytes, bytesAsText, bytesToBase64, payloadContents, payloadSize, toPayload } from './fileBytes';

const PNG_HEADER = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00]);

describe('fileBytes', () => {
	it('keeps text as text', () => {
		const svg = new TextEncoder().encode('<svg xmlns="http://www.w3.org/2000/svg"/>');
		expect(bytesAsText(svg)).toBe('<svg xmlns="http://www.w3.org/2000/svg"/>');
		expect(toPayload('static/logo.svg', svg)).toEqual({ path: 'static/logo.svg', content: '<svg xmlns="http://www.w3.org/2000/svg"/>' });
	});

	it('sends binary files as base64 and gets the same bytes back', () => {
		expect(bytesAsText(PNG_HEADER)).toBeNull();
		const payload = toPayload('static/logo.png', PNG_HEADER);
		expect(payload.encoding).toBe('base64');
		expect(payloadContents(payload)).toEqual(PNG_HEADER);
		expect(payloadSize(payload)).toBe(PNG_HEADER.length);
	});

	it('round-trips files bigger than one chunk', () => {
		const big = new Uint8Array(100_000).map((_, i) => i % 256);
		expect(base64ToBytes(bytesToBase64(big))).toEqual(big);
	});

	it('treats text with NUL bytes as binary', () => {
		expect(bytesAsText(new Uint8Array([0x61, 0x00, 0x62]))).toBeNull();
	});
});
