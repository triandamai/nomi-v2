import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { GoogleConnection } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/connections/google');
	if (!response.ok) return { google: null };
	return { google: (await response.json()) as GoogleConnection };
};

export const actions: Actions = {
	/** Starts connecting: the page sends the browser on to Google's sign-in. */
	start: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const services = data.getAll('service').map(String);
		const resume = String(data.get('resume') ?? '') || null;
		if (services.length === 0) return fail(400, { error: 'Turn on at least one service.' });
		const response = await apiFetch(fetch, cookies, '/api/connections/google/start', {
			method: 'POST',
			body: JSON.stringify({ services, resume_session_id: resume }),
		});
		if (!response.ok) {
			const text = await response.text();
			return fail(response.status, { error: text && text.length < 160 ? text : 'Couldn’t start connecting. Try again.' });
		}
		const { url } = (await response.json()) as { url: string };
		return { url };
	},
	disconnect: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/connections/google', { method: 'DELETE' });
		if (!response.ok) return fail(response.status, { error: 'Couldn’t disconnect. Try again.' });
		return { disconnected: true };
	},
};
