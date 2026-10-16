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

	it('redacts headers, connection strings and known key formats, and leaves ordinary text alone', () => {
		expect(sanitizeError('Authorization: Bearer abcdefghijklmnop123')).not.toContain('abcdefghijklmnop123');
		expect(sanitizeError('postgres://app:s3cr3t@db:5432/app')).toBe('postgres://app:[REDACTED]@db:5432/app');
		expect(sanitizeError('failed with sk-ant-api03-abcdefghijklmnopqrstuvwxyz')).not.toContain('abcdefghijklmnop');
		expect(sanitizeError('{"api_key": "hunter2-secret"}')).toBe('{"api_key": "[REDACTED]"}');
		expect(sanitizeError('The model returned 429: rate limited')).toBe('The model returned 429: rate limited');
	});

	it('handles null and empty input safely', () => {
		expect(sanitizeError(null)).toBe('');
		expect(sanitizeError(undefined)).toBe('');
		expect(sanitizeError('')).toBe('');
	});
});
