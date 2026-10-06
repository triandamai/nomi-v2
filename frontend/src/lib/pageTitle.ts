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
	'/admin/(protected)': 'Overview · Admin',
	'/admin/(protected)/agents': 'Live agents · Admin',
	'/admin/(protected)/dynamic-agents': 'Custom agents · Admin',
	'/admin/(protected)/settings/llm': 'Models · Admin',
	'/admin/(protected)/settings/embedding': 'Embeddings · Admin',
	'/admin/(protected)/users': 'Users · Admin',
	'/login': 'Log in',
	'/register': 'Sign up',
};

/** The document title: "Money · Nomi", or just "Nomi" for Home and anything unnamed. */
export function pageTitle(routeId: string | null, dataTitle?: unknown): string {
	const name =
		typeof dataTitle === 'string' && dataTitle.trim()
			? dataTitle.trim()
			: (ROUTE_TITLES[routeId ?? ''] ?? (routeId?.startsWith('/admin') ? 'Admin' : undefined));
	return name ? `${name} · Nomi` : 'Nomi';
}
