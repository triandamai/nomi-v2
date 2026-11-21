/** The pages a navigation can name while it loads ("loading projects…"). */
export type LoadingPage = 'home' | 'chat' | 'chats' | 'projects' | 'project' | 'inbox' | 'settings';

const PAGES: [pattern: RegExp, page: LoadingPage][] = [
	[/^\/$/, 'home'],
	[/^\/chat(\/|$)/, 'chat'],
	[/^\/chats(\/|$)/, 'chats'],
	[/^\/projects\/?$/, 'projects'],
	[/^\/projects\/[^/]+/, 'project'],
	[/^\/inbox(\/|$)/, 'inbox'],
	[/^\/settings(\/|$)/, 'settings'],
];

/** Which page a navigation to this path is loading, or null for one with no name of its own. */
export function loadingPage(pathname: string): LoadingPage | null {
	return PAGES.find(([pattern]) => pattern.test(pathname))?.[1] ?? null;
}

/** Whether a navigation leaves this page for another (not a change of its query or hash). */
export const opensAnotherPage = (from: string, to: string | null | undefined): boolean => Boolean(to) && to !== from;

/** How long a navigation runs before its page's placeholder shows: a quick one just swaps. */
export const LOADING_DELAY_MS = 150;
