import { describe, expect, it } from 'vitest';
import { buildAgentPlansFetchUrl } from './buildAgentPlansFetchUrl';

describe('buildAgentPlansFetchUrl', () => {
	it('builds the session-scoped path under the /chat/:sessionId route', () => {
		expect(buildAgentPlansFetchUrl('/chat/abc123', 'agent-1')).toBe('/chat/abc123/agent-plans/agent-1');
	});

	it('builds the session-scoped path under the /projects/session/:sessionId route', () => {
		expect(buildAgentPlansFetchUrl('/projects/session/abc123', 'agent-1')).toBe(
			'/projects/session/abc123/agent-plans/agent-1',
		);
	});

	it('does not drop the session ID the way WHATWG relative-URL resolution would', () => {
		// Same regression class buildMessageFetchUrl.ts guards against — plain concatenation
		// from the current page's own pathname, never `new URL(relative, base)` resolution,
		// since neither route renders with a trailing slash.
		const result = buildAgentPlansFetchUrl('/chat/abc123', 'agent-1');
		expect(result).toContain('/chat/abc123/');
		expect(result).not.toBe('/chat/agent-plans/agent-1');
	});
});
