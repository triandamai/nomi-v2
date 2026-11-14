import { fail, redirect } from '@sveltejs/kit';
import { apiFetch, apiUrl } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { Actions, PageServerLoad } from './$types';
import type { ShowcaseApp } from '$lib/types';

export type { ShowcaseApp };

// Public: anyone can browse what people built, signed in or not.
export const load: PageServerLoad = async ({ fetch, locals }) => {
	const response = await fetch(apiUrl('/api/showcase')).catch(() => null);
	const apps: ShowcaseApp[] = response?.ok ? await response.json() : [];
	return { apps, signedIn: Boolean(locals.accessToken), failed: !response?.ok };
};

export const actions: Actions = {
	// A copy of an app on show as a project of the person's own, opened right away. Signed out,
	// they sign in first.
	remix: async ({ request, cookies, fetch, locals }) => {
		if (!locals.accessToken) throw redirect(303, '/login');
		const appId = String((await request.formData()).get('app_id') ?? '');
		if (!appId) return fail(400, { remixError: m.showcase_remix_failed() });
		const response = await apiFetch(fetch, cookies, `/api/showcase/${encodeURIComponent(appId)}/remix`, { method: 'POST' });
		// No workspace picked for projects yet: Projects asks for one.
		if (response.status === 409) throw redirect(303, '/projects');
		if (!response.ok) return fail(response.status, { remixError: m.showcase_remix_failed() });
		const { session_id } = (await response.json()) as { session_id: string };
		throw redirect(303, `/projects/session/${session_id}`);
	},
};
