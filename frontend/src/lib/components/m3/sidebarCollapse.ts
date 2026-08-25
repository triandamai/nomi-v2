import { browser } from '$app/environment';

const COMPACT_BREAKPOINT_PX = 600;

// Always call from onMount — reading localStorage/matchMedia during SSR or the
// initial client render would produce a value that disagrees with the SSR'd
// markup and trigger a hydration mismatch. Callers render collapsed=false
// (the SSR-safe default) until onMount updates it.
export function readInitialCollapsed(storageKey: string): boolean {
	if (!browser) return false;
	const stored = localStorage.getItem(storageKey);
	if (stored !== null) return stored === 'true';
	return window.matchMedia(`(max-width: ${COMPACT_BREAKPOINT_PX}px)`).matches;
}

export function persistCollapsed(storageKey: string, collapsed: boolean): void {
	if (!browser) return;
	localStorage.setItem(storageKey, String(collapsed));
}
