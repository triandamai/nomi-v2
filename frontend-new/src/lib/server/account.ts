import { isLocale } from '$lib/paraglide/runtime';
import type { OrgSummary } from '$lib/adapters/api';
import { apiFetch, apiUrl, type ApiEvent } from './api';
import { readApiError } from './apiError';
import { setLanguageCookie } from './locale';
import { REFRESH_COOKIE, clearSession, setSession } from './session';
import { safeRedirect } from './signIn';
import { setThemeCookie } from './theme';

// What the account menu and the workspace switcher change: signing out, the theme and language,
// the workspace projects are listed from, and new workspaces. Each goes to the backend first and
// changes this device only once the backend has it.

/** Ends this device's sign-in: the refresh token is revoked on the backend (best effort) and the
 * session cookies go either way. */
export async function signOut(event: ApiEvent): Promise<void> {
	const refreshToken = event.cookies.get(REFRESH_COOKIE);
	if (refreshToken) {
		await event
			.fetch(apiUrl('/api/auth/logout'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ refresh_token: refreshToken }) })
			.catch(() => null);
	}
	clearSession(event.cookies);
}

/** Where signing out lands: sign in, or a page on this site it was asked to go back to. */
export function signOutTarget(redirectTo: string | null): string {
	const target = safeRedirect(redirectTo);
	return target === '/' ? '/sign-in' : target;
}

type Outcome = { ok: true } | { ok: false; status: number; message?: string };

async function outcome(response: Response): Promise<Outcome> {
	if (response.ok) return { ok: true };
	const error = await readApiError(response);
	return { ok: false, status: error.status, ...(error.message && { message: error.message }) };
}

const THEMES = new Set(['light', 'dark', 'system']);

/** A zone this runtime's Intl knows ("Asia/Jakarta"), the same list the time zone picker offers. */
function knownZone(zone: unknown): zone is string {
	if (typeof zone !== 'string' || !zone) return false;
	try {
		new Intl.DateTimeFormat('en', { timeZone: zone });
		return true;
	} catch {
		return false;
	}
}

/** An `agent_type` as the backend names one ("money", "dyn_tax_helper"), or '' for Nomi. */
const AGENT_TYPE = /^[a-z][a-z0-9_]{0,63}$|^$/;

export type PreferenceChange = { theme?: unknown; language?: unknown; timezone?: unknown; default_agent?: unknown; enter_sends?: unknown };

/** Saves the theme, the language, the time zone, who new chats start with or whether Enter sends,
 * with the person's preferences; the theme and the language are kept on this device too. */
export async function savePreferences(event: ApiEvent, change: PreferenceChange): Promise<Outcome> {
	const theme = typeof change.theme === 'string' && THEMES.has(change.theme) ? (change.theme as 'light' | 'dark' | 'system') : undefined;
	const language = typeof change.language === 'string' && isLocale(change.language) ? change.language : undefined;
	const timezone = knownZone(change.timezone) ? change.timezone : undefined;
	const agent = typeof change.default_agent === 'string' && AGENT_TYPE.test(change.default_agent) ? change.default_agent : undefined;
	const enterSends = typeof change.enter_sends === 'boolean' ? change.enter_sends : undefined;
	const unknown =
		(change.theme !== undefined && !theme) ||
		(change.language !== undefined && !language) ||
		(change.timezone !== undefined && !timezone) ||
		(change.default_agent !== undefined && agent === undefined) ||
		(change.enter_sends !== undefined && enterSends === undefined);
	if (unknown || (!theme && !language && !timezone && agent === undefined && enterSends === undefined)) return { ok: false, status: 400 };

	const response = await apiFetch(event, '/api/preferences', {
		method: 'PUT',
		body: JSON.stringify({
			...(theme && { theme }),
			...(language && { language }),
			...(timezone && { timezone }),
			...(agent !== undefined && { default_agent: agent }),
			...(enterSends !== undefined && { enter_sends: enterSends }),
		}),
	});
	if (!response.ok) return { ok: false, status: response.status };
	if (theme) setThemeCookie(event.cookies, theme);
	if (language) setLanguageCookie(event.cookies, language);
	return { ok: true };
}

/** Switches the person's active workspace (`PUT /api/auth/active-org`) and keeps the new access
 * token on this device. */
export async function switchActiveOrg(event: ApiEvent, orgId: string): Promise<Outcome> {
	const response = await apiFetch(event, '/api/auth/active-org', { method: 'PUT', body: JSON.stringify({ org_id: orgId }) });
	if (!response.ok) return outcome(response);
	const { access_token } = (await response.json()) as { access_token: string };
	event.locals.accessToken = access_token;
	setSession(event.cookies, { access_token });
	return { ok: true };
}

/** Picks the workspace projects are listed from and started in, switching the active workspace too. */
export async function chooseProjectsWorkspace(event: ApiEvent, orgId: string): Promise<Outcome> {
	const switched = await switchActiveOrg(event, orgId);
	if (!switched.ok) return switched;
	return outcome(await apiFetch(event, '/api/preferences/project-org', { method: 'PUT', body: JSON.stringify({ org_id: orgId }) }));
}

/** The backend's limit on a workspace's name (backend: routes/orgs.rs MAX_NAME_CHARS). */
export const WORKSPACE_NAME_MAX = 80;

export type Created = { ok: true; org: OrgSummary } | { ok: false; status: number; message?: string; field?: 'required' | 'long' };

/** A new workspace, with the person as its owner. */
export async function createWorkspace(event: ApiEvent, rawName: string): Promise<Created> {
	const name = rawName.trim();
	if (!name) return { ok: false, status: 400, field: 'required' };
	if ([...name].length > WORKSPACE_NAME_MAX) return { ok: false, status: 400, field: 'long' };
	const response = await apiFetch(event, '/api/orgs', { method: 'POST', body: JSON.stringify({ name }) });
	if (!response.ok) return (await outcome(response)) as Created;
	return { ok: true, org: (await response.json()) as OrgSummary };
}
