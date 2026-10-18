// Recent chats listed in the navigation drawer (Preferences → Navigation drawer).

import { m } from '$lib/paraglide/messages';
import type { Preferences, SessionSummary } from '$lib/types';

/** The sizes Preferences offers; the backend accepts only these. */
export const RECENTS_LIMITS = [5, 10, 15, 20] as const;
export type RecentsLimit = (typeof RECENTS_LIMITS)[number];
export const DEFAULT_RECENTS_LIMIT: RecentsLimit = 10;

export interface RecentsSettings {
	enabled: boolean;
	limit: RecentsLimit;
}

export interface RecentChat {
	id: string;
	title: string;
	href: string;
}

export type RecentsState = { status: 'loading' } | { status: 'error' } | { status: 'ready'; chats: RecentChat[] };

export function isRecentsLimit(value: unknown): value is RecentsLimit {
	return (RECENTS_LIMITS as readonly unknown[]).includes(value);
}

/** The saved settings, or the defaults (shown, ten at a time) until the person changes them. */
export function resolveRecents(preferences: Pick<Preferences, 'drawer_recents' | 'drawer_recents_limit'>): RecentsSettings {
	return {
		enabled: preferences.drawer_recents !== false,
		limit: isRecentsLimit(preferences.drawer_recents_limit) ? preferences.drawer_recents_limit : DEFAULT_RECENTS_LIMIT,
	};
}

/** The newest `limit` chats (sessions come newest first). Project sessions live under Projects. */
export function toRecentChats(sessions: SessionSummary[], limit: number): RecentChat[] {
	return sessions
		.filter((session) => !session.project_id)
		.slice(0, limit)
		.map((session) => ({ id: session.id, title: session.title ?? m.nav_new_chat(), href: `/chat/${session.id}` }));
}

export function isRecentChatActive(pathname: string, id: string): boolean {
	return pathname === `/chat/${id}`;
}

/**
 * Whether a finished navigation should reload the list: only a move to another page. The first
 * load happens on mount instead, because afterNavigate doesn't fire for a component that mounts
 * after the page has loaded (the phone drawer or the expanded rail).
 */
export function refreshesAfterNavigation(from: { url: URL } | null, to: { url: URL } | null): boolean {
	return !!from && !!to && from.url.pathname !== to.url.pathname;
}

/** Reads the person's sessions through the app's own endpoint (routes/(app)/chats/recent). */
export async function loadRecentChats(fetchFn: typeof fetch, limit: number): Promise<RecentsState> {
	try {
		const response = await fetchFn('/chats/recent');
		if (!response.ok) return { status: 'error' };
		const { sessions } = (await response.json()) as { sessions: SessionSummary[] };
		return { status: 'ready', chats: toRecentChats(sessions, limit) };
	} catch {
		return { status: 'error' };
	}
}

/** The Preferences form's switch and size, or null when the size isn't one it offers. */
export function parseRecentsForm(data: FormData): Pick<Preferences, 'drawer_recents' | 'drawer_recents_limit'> | null {
	const limit = Number(data.get('drawer_recents_limit'));
	if (!isRecentsLimit(limit)) return null;
	return { drawer_recents: data.get('drawer_recents') === 'true', drawer_recents_limit: limit };
}
