import { redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { Cookies } from '@sveltejs/kit';

const API_URL = env.API_URL ?? 'http://localhost:8080';

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
	cookies.set('access_token', access_token, { httpOnly: true, path: '/', sameSite: 'lax' });

	return fetchFn(apiUrl(path), withAuth(init, access_token));
}
