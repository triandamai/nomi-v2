import { redirect, type Cookies } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { ACCESS_COOKIE, REFRESH_COOKIE, clearSession, needsRefresh, setSession } from './session';

/** Where someone goes when their sign-in can't be renewed. */
export const SIGN_IN_PATH = '/sign-in';

/** What a server load or action needs to call the backend as the person. */
export type ApiEvent = { fetch: typeof fetch; cookies: Cookies; locals: App.Locals };

const apiBase = () => env.API_URL ?? 'http://localhost:8080';

export function apiUrl(path: string): string {
	return `${apiBase()}${path}`;
}

/** Whether `url` is a call to the backend API. */
export function isApiRequest(url: string): boolean {
	return url.startsWith(`${apiBase()}/`);
}

async function requestNewAccessToken(event: ApiEvent): Promise<string | null> {
	const refreshToken = event.cookies.get(REFRESH_COOKIE);
	if (!refreshToken) return null;
	const response = await event
		.fetch(apiUrl('/api/auth/refresh'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ refresh_token: refreshToken }),
		})
		.catch(() => null);
	if (!response?.ok) {
		// A refresh token the API no longer takes is dropped, so it isn't tried on every request.
		if (response?.status === 401) clearSession(event.cookies);
		return null;
	}
	const { access_token } = (await response.json()) as { access_token: string };
	// The refresh token's expiry moved out on the API: the cookies follow.
	setSession(event.cookies, { access_token, refresh_token: refreshToken });
	return access_token;
}

/**
 * A new access token, at most once per request: every load and call in one page request shares
 * the same refresh (a page load makes several calls at once, and each used to refresh on its own).
 */
export function refreshOnce(event: ApiEvent): Promise<string | null> {
	event.locals.refreshing ??= requestNewAccessToken(event).then((token) => {
		event.locals.accessToken = token ?? undefined;
		return token;
	});
	return event.locals.refreshing;
}

/** Signs the person in for this request: their token, renewed first when it's about to expire. */
export async function authenticate(event: ApiEvent): Promise<void> {
	const token = event.cookies.get(ACCESS_COOKIE);
	const canRefresh = Boolean(event.cookies.get(REFRESH_COOKIE));
	event.locals.accessToken = canRefresh && needsRefresh(token) ? ((await refreshOnce(event)) ?? undefined) : token;
}

function withAuth(init: RequestInit | undefined, accessToken: string | undefined): RequestInit {
	return {
		...init,
		headers: {
			'Content-Type': 'application/json',
			...(init?.headers ?? {}),
			...(accessToken ? { Authorization: `Bearer ${accessToken}` } : {}),
		},
	};
}

/** Calls the backend as the person. A 401 renews the sign-in once and retries; when that fails
 * too, they're signed out and sent to sign in. */
export async function apiFetch(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	const sentWith = event.locals.accessToken;
	const response = await event.fetch(apiUrl(path), withAuth(init, sentWith));
	if (response.status !== 401) return response;

	// Another call in this request may have renewed it already.
	const renewed = event.locals.accessToken !== sentWith && event.locals.accessToken ? event.locals.accessToken : await refreshOnce(event);
	if (!renewed) {
		clearSession(event.cookies);
		redirect(303, SIGN_IN_PATH);
	}
	return event.fetch(apiUrl(path), withAuth(init, renewed));
}
