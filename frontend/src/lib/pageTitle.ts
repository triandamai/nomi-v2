import { m } from '$lib/paraglide/messages';

/** What each page is called, by route id. Pages that load their own `title` (a chat, a Home
 * section) use that instead. */
const admin = (page: () => string) => () => m.title_admin_suffix({ page: page() });
const ROUTE_TITLES: Record<string, () => string> = {
	'/(app)/account': m.title_account,
	'/(app)/chat/[sessionId]': m.chat_title,
	'/(app)/chats': m.chats_title,
	'/(app)/connections': m.conn_title,
	'/(app)/crew': m.home_your_crew,
	'/(app)/memory': m.mem_title,
	'/(app)/models': m.title_models,
	'/(app)/money': m.money_title,
	'/(app)/preferences': m.prefs_title,
	'/(app)/profile': m.profile_title,
	'/(app)/projects': m.projects_title,
	'/(app)/projects/session/[sessionId]': m.project_context,
	'/(app)/reminders': m.rem_title,
	'/admin/(protected)': admin(m.admin_overview),
	'/admin/(protected)/agents': admin(m.admin_live_agents),
	'/admin/(protected)/dynamic-agents': admin(m.admin_custom_agents),
	'/admin/(protected)/settings/llm': admin(m.admin_models),
	'/admin/(protected)/settings/embedding': admin(m.admin_embeddings),
	'/admin/(protected)/users': admin(m.admin_users),
	'/login': m.login_submit,
	'/register': m.title_sign_up,
};

/** The document title: "Money · Nomi", or just "Nomi" for Home and anything unnamed. */
export function pageTitle(routeId: string | null, dataTitle?: unknown): string {
	const name =
		typeof dataTitle === 'string' && dataTitle.trim()
			? dataTitle.trim()
			: (ROUTE_TITLES[routeId ?? '']?.() ?? (routeId?.startsWith('/admin') ? m.admin_tag() : undefined));
	return name ? `${name} · Nomi` : 'Nomi';
}
