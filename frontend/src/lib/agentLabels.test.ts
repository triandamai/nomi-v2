import { describe, expect, it } from 'vitest';
import { sanitizeError } from './agentLabels';

describe('sanitizeError', () => {
	it('redacts query parameters containing key, token, or secret', () => {
		const raw =
			'llm call failed: http request failed: error sending request for url (https://generativelanguage.googleapis.com/v1beta/models/gemini-pro-latest:streamGenerateContent?alt=sse&key=AQ.Ab8xxredactedxx)';
		const sanitized = sanitizeError(raw);
		expect(sanitized).not.toContain('AQ.Ab8xxredactedxx');
		expect(sanitized).toContain('key=[REDACTED]');
	});

	it('handles multiple sensitive parameters', () => {
		const raw = 'error at https://example.com/api?token=secret123&other=safe&api_key=secret456';
		const sanitized = sanitizeError(raw);
		expect(sanitized).toContain('token=[REDACTED]');
		expect(sanitized).toContain('api_key=[REDACTED]');
		expect(sanitized).toContain('other=safe');
	});

	it('handles null and empty input safely', () => {
		expect(sanitizeError(null)).toBe('');
		expect(sanitizeError(undefined)).toBe('');
		expect(sanitizeError('')).toBe('');
	});
});
