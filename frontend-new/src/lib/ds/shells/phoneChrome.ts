import { getContext, setContext } from 'svelte';
import type { Tab } from '../chrome/TabBar.svelte';

// How AppShell dresses a screen on a phone (below 768), as Flow / Mobile Chat draws it: a top-level
// screen has the tab bar along the bottom in place of the drawer ("Chats is a tab, not a drawer"),
// and one with a large title of its own (Chats) leaves the top bar out. Set by whoever renders the
// shell, as mainLayout is, so AppShell's props stay the same as the old app's twin of it.

export type PhoneChrome = {
	/** On a phone now (the window's width, or the server's guess before the page runs). */
	readonly phone: boolean;
	/** The tab bar's label and tabs; null on a screen inside a tab (a chat), which keeps the bar. */
	readonly tabs: { label: string; tabs: Tab[] } | null;
	/** The screen draws its own top on a phone (a large title, a chat's nav bar): no top bar. */
	readonly ownTitle: boolean;
};

const KEY = Symbol('app-shell-phone-chrome');

export const setPhoneChrome = (chrome: PhoneChrome) => setContext(KEY, chrome);

export const phoneChrome = (): PhoneChrome => getContext<PhoneChrome | undefined>(KEY) ?? { phone: false, tabs: null, ownTitle: false };
