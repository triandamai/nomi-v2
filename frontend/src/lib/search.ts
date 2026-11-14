// The app's search (⌘K): what the box asks the backend (through /search, the app's own
// proxy for GET /api/search) and where each hit opens.
import type { SearchKind, SearchResult } from '$lib/types';

export const SEARCH_KINDS: SearchKind[] = ['chat', 'memory', 'file'];
/** How many hits the dialog shows. */
export const SEARCH_LIMIT = 20;
/** How long typing pauses before the search runs. */
export const SEARCH_DEBOUNCE_MS = 200;

/** ⌘K on a Mac, Ctrl+K elsewhere. */
export function isSearchShortcut(event: Pick<KeyboardEvent, 'key' | 'metaKey' | 'ctrlKey' | 'altKey' | 'shiftKey'>): boolean {
	return event.key.toLowerCase() === 'k' && (event.metaKey || event.ctrlKey) && !event.altKey && !event.shiftKey;
}

/** The app's search address for `query`, or null when there's nothing to look for. */
export function searchUrl(query: string, kinds: SearchKind[] = SEARCH_KINDS, limit = SEARCH_LIMIT): string | null {
	const q = query.trim();
	if (!q) return null;
	const params = new URLSearchParams({ q, limit: String(limit) });
	if (kinds.length < SEARCH_KINDS.length) params.set('kinds', kinds.join(','));
	return `/search?${params}`;
}

/** Where a hit opens: its chat (a project's own in the project), Memory, or the file's project. */
export function searchHref(result: SearchResult): string {
	switch (result.kind) {
		case 'chat':
			return result.project_id ? `/projects/session/${result.session_id}` : `/chat/${result.session_id}`;
		case 'memory':
			return '/memory';
		case 'file':
			return `/projects/session/${result.session_id}`;
	}
}
