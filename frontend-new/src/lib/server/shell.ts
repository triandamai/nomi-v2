import { isRedirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { getLocale, isLocale } from '$lib/paraglide/runtime';
import { DEVICE_COOKIE } from '$lib/device';
import { THEME_COOKIE, parseTheme, type Theme } from '$lib/theme';
import type { CrewMember, OrgList, Preferences, Profile, SessionSummary, UsageBrief } from '$lib/adapters/api';
import {
	accountLine,
	chatsCount,
	crewRows,
	crewSummary,
	firstName,
	personName,
	projectsWorkspace,
	recents,
	usageCard,
	workspaceRows,
	type CrewRowProps,
	type Recent,
	type WorkspaceRow,
} from '$lib/adapters/shell';
import { apiFetch, type ApiEvent } from './api';
import { registerDevice } from './device';
import { setLanguageCookie } from './locale';
import { setThemeCookie } from './theme';

/** Everything the app shell shows around a page, in the design system's terms. */
export type Shell = {
	person: { name: string; firstName: string; avatar: string | null };
	theme: Theme;
	/** The person's saved time zone; null when they never saved one (the browser's decides). */
	timeZone: string | null;
	usage: { card: { plan: string; used: number; caption: string }; account: string; canUpgrade: boolean } | null;
	unread: number;
	recents: Recent[];
	chats: number;
	crew: { rows: CrewRowProps[]; summary: { kind: 'idle' | 'live'; label: string } };
	workspaces: WorkspaceRow[];
	/** The workspace projects are listed from: the Build group's chip and Projects count. */
	projectsWorkspace: { id: string; name: string; projects: number } | null;
	/** This browser's device id (an httpOnly cookie): what a sign-out from elsewhere names. */
	deviceId: string | null;
	/** When this was loaded, so the page and its hydration start from the same minute. */
	now: string;
};

/** A call that failed is left out, except a sign-in that can't be renewed: apiFetch's redirect to
 * sign in goes through. */
function unlessSignedOut(error: unknown): null {
	if (isRedirect(error)) throw error;
	return null;
}

/** The body of a call that worked, else null: a part that can't be loaded is left out, not fatal. */
async function json<T>(response: Response | null): Promise<T | null> {
	if (!response?.ok) return null;
	return (await response.json().catch(() => null)) as T | null;
}

/**
 * Until someone picks a workspace for projects, the API lists none and won't start one. The
 * shell's Build group always names one, so the person's first workspace (the one they signed up
 * with) is picked for them; the switcher changes it.
 */
async function settleProjectsWorkspace(event: ApiEvent, list: OrgList): Promise<OrgList> {
	const first = list.orgs[0];
	if (list.project_org_id || !first) return list;
	const response = await apiFetch(event, '/api/preferences/project-org', { method: 'PUT', body: JSON.stringify({ org_id: first.id }) }).catch(unlessSignedOut);
	return response?.ok ? { ...list, project_org_id: first.id } : list;
}

export async function loadShell(event: ApiEvent & { userAgent: string }, now: Date = new Date()): Promise<Shell> {
	const get = (path: string) => apiFetch(event, path).catch(unlessSignedOut);
	const [profile, preferences, agents, usage, unread, sessions, orgs] = await Promise.all([
		get('/api/profile').then(json<Profile>),
		get('/api/preferences').then(json<Preferences>),
		get('/api/agents').then(json<{ agents: CrewMember[] }>),
		get('/api/usage/brief').then(json<UsageBrief>),
		get('/api/notifications/unread').then(json<{ unread: number }>),
		get('/api/sessions').then(json<{ sessions: SessionSummary[] }>),
		get('/api/orgs').then(json<OrgList>),
		registerDevice(event, event.userAgent),
	]);

	// The saved language and theme win on every device; the cookies make the next render use them.
	if (preferences && isLocale(preferences.language) && preferences.language !== getLocale()) setLanguageCookie(event.cookies, preferences.language);
	const theme = parseTheme(preferences?.theme ?? event.cookies.get(THEME_COOKIE));
	if (preferences && event.cookies.get(THEME_COOKIE) !== theme) setThemeCookie(event.cookies, theme);

	const workspaces = orgs ? await settleProjectsWorkspace(event, orgs) : null;
	const chosen = workspaces ? projectsWorkspace(workspaces) : null;
	const timeZone = preferences?.has_stored_timezone ? preferences.timezone : null;
	const name = profile ? personName(profile) : m.shell_you();
	const members = agents?.agents ?? [];

	return {
		person: { name, firstName: firstName(name), avatar: profile?.avatar_url ?? null },
		theme,
		timeZone,
		usage: usage ? { card: usageCard(usage), account: accountLine(usage), canUpgrade: usage.can_upgrade } : null,
		unread: unread?.unread ?? 0,
		recents: recents(sessions?.sessions ?? [], now, undefined, undefined, timeZone ?? undefined),
		chats: chatsCount(sessions?.sessions ?? []),
		crew: { rows: crewRows(members), summary: crewSummary(members) },
		workspaces: workspaces ? workspaceRows(workspaces) : [],
		projectsWorkspace: chosen && { id: chosen.id, name: chosen.name, projects: chosen.projects },
		deviceId: event.cookies.get(DEVICE_COOKIE) ?? null,
		now: now.toISOString(),
	};
}
