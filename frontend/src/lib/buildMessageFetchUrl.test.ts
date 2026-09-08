import { describe, expect, it } from 'vitest';
import { buildMessageFetchUrl } from './buildMessageFetchUrl';

describe('buildMessageFetchUrl', () => {
	it('builds the session-scoped path under the /chat/:sessionId route', () => {
		expect(buildMessageFetchUrl('/chat/abc123', 'msg-1')).toBe('/chat/abc123/message/msg-1');
	});

	it('builds the session-scoped path under the /projects/session/:sessionId route', () => {
		expect(buildMessageFetchUrl('/projects/session/abc123', 'msg-1')).toBe(
			'/projects/session/abc123/message/msg-1',
		);
	});

	it('does not drop the session ID the way WHATWG relative-URL resolution would', () => {
		// Regression check for the bug this function replaces: `new URL('message/x', base)`
		// against a base with no trailing slash drops the base's last path segment (the session
		// ID) instead of appending to it. Plain concatenation must never do that.
		const result = buildMessageFetchUrl('/chat/abc123', 'msg-1');
		expect(result).toContain('/chat/abc123/');
		expect(result).not.toBe('/chat/message/msg-1');
	});
});
