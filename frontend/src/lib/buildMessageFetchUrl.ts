/** Builds the absolute path for fetching one patched message, from the current page's own
 * pathname. `ChatThread.svelte` is mounted under two different route trees — `/chat/[sessionId]`
 * and `/projects/session/[sessionId]` — each with its own sibling `message/[messageId]` proxy
 * route, so the fetch target must be derived from wherever the component is actually mounted
 * rather than hardcoding either tree.
 *
 * Deliberately plain string concatenation, not `new URL(relative, base)` resolution: neither
 * route renders with a trailing slash (both are SvelteKit leaf pages, no `trailingSlash` config
 * exists in this app, and every link into them omits the trailing slash), so a leading-slash-less
 * relative reference like `fetch('message/xyz')` would have its base's last path segment (the
 * session ID) silently dropped by WHATWG URL resolution instead of appended to. */
export function buildMessageFetchUrl(pathname: string, messageId: string): string {
	return `${pathname}/message/${messageId}`;
}
