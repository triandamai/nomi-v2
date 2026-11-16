// The app shell's keyboard shortcuts: the ones the sidebar, the account menu and the workspace
// switcher show next to their rows, and the shortcuts dialog lists. ⌘ on a Mac, Ctrl elsewhere.
// Escape closes menus and dialogs on its own (they're native popovers and dialogs).

export type Shortcut = 'search' | 'new-chat' | 'settings' | 'workspace-settings' | 'shortcuts';

/** The keys as the design writes them (⌘ even off a Mac; the dialog says so). */
export const SHORTCUT_KEYS: Record<Shortcut, string> = {
	// The top bar's search wears ⌘K (App / Home), as the old app's did; so New chat, whose sidebar
	// row the pen also marks ⌘K, takes ⌘⇧O (AGENTS.md §9).
	search: '⌘K',
	'new-chat': '⌘ ⇧ O',
	settings: '⌘ ,',
	'workspace-settings': '⌘ ⇧ ,',
	shortcuts: '?',
};

type Press = { key: string; code: string; metaKey: boolean; ctrlKey: boolean; shiftKey: boolean; altKey: boolean; /** In a field, where ? is a character. */ typing: boolean };

export function shortcutFor(press: Press): Shortcut | null {
	const command = (press.metaKey || press.ctrlKey) && !press.altKey;
	if (command && press.code === 'KeyK' && !press.shiftKey) return 'search';
	if (command && press.code === 'KeyO' && press.shiftKey) return 'new-chat';
	if (command && press.code === 'Comma') return press.shiftKey ? 'workspace-settings' : 'settings';
	if (!command && !press.altKey && press.key === '?' && !press.typing) return 'shortcuts';
	return null;
}

/** Whether a key goes to something someone types in. */
export function isTyping(target: EventTarget | null): boolean {
	if (!(target instanceof HTMLElement)) return false;
	return target.isContentEditable || target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement;
}
