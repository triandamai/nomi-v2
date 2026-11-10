import { fail, redirect } from '@sveltejs/kit';
import type { Alert, Fields } from '$lib/auth/problems';
import { withRedirect } from '$lib/auth/redirects';
import { apiFetch } from '$lib/server/api';
import { safeRedirect } from '$lib/server/signIn';
import type { Actions, PageServerLoad } from './$types';

// First run (Build → First run, jPO7K): the one screen after a new account's code. Its name, then
// on to where it was going (an invite, or home).

export const load: PageServerLoad = async (event) => {
	if (!event.locals.accessToken) redirect(303, withRedirect('/sign-in', `${event.url.pathname}${event.url.search}`));
	const response = await apiFetch(event, '/api/profile').catch(() => null);
	const profile = response?.ok ? ((await response.json()) as { display_name?: string | null; email?: string }) : {};
	// The name when we know it: what they set (Google gives one), or the address before the @.
	const name = profile.display_name || (profile.email ?? '').split('@')[0] || '';
	// FirstRun has its own mascot: no greeting above it.
	return { name, greeting: false as const };
};

const failure = (status: number, name: string, fields: Fields, alert: Alert | null = null) => fail(status, { name, fields, alert });

export const actions: Actions = {
	default: async (event) => {
		const name = String((await event.request.formData()).get('name') ?? '').trim();
		if (!name) return failure(400, name, { name: 'name_required' });
		const response = await apiFetch(event, '/api/profile', { method: 'PUT', body: JSON.stringify({ display_name: name }) });
		if (!response.ok) return failure(response.status, name, {}, { kind: 'firstrun_failed', status: response.status });
		redirect(303, safeRedirect(event.url.searchParams.get('redirect_to')));
	},
};
