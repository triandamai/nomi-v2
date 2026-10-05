import { fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Profile } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export interface SignInMethods {
	configured: boolean;
	password: boolean;
	google_email: string | null;
}

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [profileResponse, methodsResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/profile'),
		apiFetch(fetch, cookies, '/api/auth/google'),
	]);
	const profile: Profile | null = profileResponse.ok ? ((await profileResponse.json()) as Profile) : null;
	const methods: SignInMethods | null = methodsResponse.ok ? ((await methodsResponse.json()) as SignInMethods) : null;
	return { profile, methods };
};

export const actions: Actions = {
	/** Adds Google sign-in to this account: off to Google. */
	link: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/auth/google/link', { method: 'POST' });
		if (!response.ok) return fail(response.status, { error: 'Couldn’t start linking Google. Try again.' });
		const { url } = (await response.json()) as { url: string };
		redirect(303, url);
	},
	unlink: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/auth/google', { method: 'DELETE' });
		if (!response.ok) return fail(response.status, { error: (await response.text()) || 'Couldn’t unlink Google.' });
		return { unlinked: true };
	},
};
