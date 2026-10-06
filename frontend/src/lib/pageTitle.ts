/** What each page is called, by route id. Pages that load their own `title` (a chat, a Home
 * section) use that instead. */
const ROUTE_TITLES: Record<string, string> = {
	'/(app)/account': 'Account',
	'/(app)/chat/[sessionId]': 'Chat',
	'/(app)/chats': 'Chats',
	'/(app)/connections': 'Connections',
	'/(app)/crew': 'Your crew',
	'/(app)/memory': 'Memory',
	'/(app)/models': 'Models',
	'/(app)/money': 'Money',
	'/(app)/preferences': 'Preferences',
	'/(app)/profile': 'Profile',
	'/(app)/projects': 'Projects',
	'/(app)/projects/session/[sessionId]': 'Project',
	'/(app)/reminders': 'Reminders',
	'/login': 'Log in',
	'/register': 'Sign up',
};

/** The document title: "Money · Nomi", or just "Nomi" for Home and anything unnamed. */
export function pageTitle(routeId: string | null, dataTitle?: unknown): string {
	const name =
		typeof dataTitle === 'string' && dataTitle.trim()
			? dataTitle.trim()
			: routeId?.startsWith('/admin')
				? 'Admin'
				: ROUTE_TITLES[routeId ?? ''];
	return name ? `${name} · Nomi` : 'Nomi';
}
