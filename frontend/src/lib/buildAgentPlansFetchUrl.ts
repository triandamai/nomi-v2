/** Builds the absolute path for fetching one agent_session's plan version history, from the
 * current page's own pathname — same reasoning as buildMessageFetchUrl.ts: ChatThread.svelte
 * (and, transitively, PlanBlock.svelte) is mounted under two different route trees
 * (`/chat/[sessionId]` and `/projects/session/[sessionId]`), each with its own sibling
 * `agent-plans/[agentSessionId]` proxy route, and neither renders with a trailing slash — so
 * this is plain string concatenation, never `new URL(relative, base)` resolution. */
export function buildAgentPlansFetchUrl(pathname: string, agentSessionId: string): string {
	return `${pathname}/agent-plans/${agentSessionId}`;
}
