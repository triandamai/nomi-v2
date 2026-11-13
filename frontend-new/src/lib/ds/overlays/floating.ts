// Where a panel that floats off a button goes, and how the arrow keys move through its items:
// shared by the menus that bring their own panel (the account menu, the workspace switcher).
// Menu.svelte does the same for the standard menu.

type Box = { top: number; bottom: number; left: number; right: number };
type Size = { width: number; height: number };

/** The gap between the button and its panel, and the panel's least distance from the window's edge. */
const GAP = 4;
const MARGIN = 8;

/** Below the button when it fits, else above; its start (or end) edge on the button's. */
export function placePanel(anchor: Box, panel: Size, viewport: Size, align: 'start' | 'end' = 'start'): { top: number; left: number } {
	const below = anchor.bottom + GAP + panel.height <= viewport.height;
	const left = align === 'end' ? anchor.right - panel.width : anchor.left;
	return {
		top: below ? anchor.bottom + GAP : Math.max(MARGIN, anchor.top - GAP - panel.height),
		left: Math.min(Math.max(MARGIN, left), viewport.width - panel.width - MARGIN),
	};
}

/** The item a key moves focus to (arrows wrap, Home and End jump), or null for any other key. */
export function nextIndex(key: string, at: number, count: number): number | null {
	if (count === 0) return null;
	const target = { ArrowDown: at + 1, ArrowUp: at < 0 ? count - 1 : at - 1, Home: 0, End: count - 1 }[key];
	return target === undefined ? null : (target + count) % count;
}

/** A menu's items in order: menuitem, menuitemradio and menuitemcheckbox alike. */
export function menuItems(menu: HTMLElement | undefined): HTMLElement[] {
	return [...(menu?.querySelectorAll<HTMLElement>('[role^="menuitem"]') ?? [])];
}
