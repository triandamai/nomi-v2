import { redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { Cookies } from '@sveltejs/kit';
import { clearSession, setSession } from '$lib/server/sessionCookies';

const API_URL = env.API_URL ?? 'http://localhost:8080';

/** Whether `url` is a call to the backend API. */
export function isApiRequest(url: string): boolean {
	return url.startsWith(`${API_URL}/`);
}

export function apiUrl(path: string): string {
	return `${API_URL}${path}`;
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

export async function apiFetch(
	fetchFn: typeof fetch,
	cookies: Cookies,
	path: string,
	init?: RequestInit,
): Promise<Response> {
	const accessToken = cookies.get('access_token');
	const response = await fetchFn(apiUrl(path), withAuth(init, accessToken));

	if (response.status !== 401) {
		return response;
	}

	const refreshToken = cookies.get('refresh_token');
	if (!refreshToken) {
		cookies.delete('access_token', { path: '/' });
		throw redirect(303, '/login');
	}

	const refreshResponse = await fetchFn(apiUrl('/api/auth/refresh'), {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ refresh_token: refreshToken }),
	});

	if (!refreshResponse.ok) {
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });
		throw redirect(303, '/login');
	}

	const { access_token } = (await refreshResponse.json()) as { access_token: string };
	// The refresh token's expiry moved out on the API: the cookies follow.
	setSession(cookies, { access_token, refresh_token: refreshToken });

	return fetchFn(apiUrl(path), withAuth(init, access_token));
}

/** A new access token from the refresh token, or null when it's gone or no longer valid. */
export async function refreshSession(fetchFn: typeof fetch, cookies: Cookies): Promise<string | null> {
	const refreshToken = cookies.get('refresh_token');
	if (!refreshToken) return null;
	const response = await fetchFn(apiUrl('/api/auth/refresh'), {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ refresh_token: refreshToken }),
	}).catch(() => null);
	if (!response) return null;
	if (!response.ok) {
		// A refresh token the API no longer accepts is dropped, so it isn't tried on every request.
		if (response.status === 401) clearSession(cookies);
		return null;
	}
	const { access_token } = (await response.json()) as { access_token: string };
	setSession(cookies, { access_token, refresh_token: refreshToken });
	return access_token;
}
