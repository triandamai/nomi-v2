import { fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { NotificationItem } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

interface InboxPage {
	items: NotificationItem[];
	unread: number;
	next_before: string | null;
}

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const before = url.searchParams.get('before');
	const query = before ? `?before=${encodeURIComponent(before)}` : '';
	const [inboxResponse, prefsResponse] = await Promise.all([
		apiFetch(fetch, cookies, `/api/notifications${query}`),
		apiFetch(fetch, cookies, '/api/notifications/preferences'),
	]);
	const inbox: InboxPage = inboxResponse.ok ? ((await inboxResponse.json()) as InboxPage) : { items: [], unread: 0, next_before: null };
	const preferences = prefsResponse.ok
		? ((await prefsResponse.json()) as { email_account: boolean; email_promos: boolean })
		: { email_account: true, email_promos: true };
	return { inbox, preferences, older: Boolean(before) };
};

/** Only links inside the app are followed. */
function safeLink(link: FormDataEntryValue | null): string | null {
	return typeof link === 'string' && link.startsWith('/') && !link.startsWith('//') ? link : null;
}

export const actions: Actions = {
	/** Marks one read, then goes where it points (if anywhere). */
	open: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') return fail(400);
		await apiFetch(fetch, cookies, `/api/notifications/${id}/read`, { method: 'POST' });
		const link = safeLink(data.get('link'));
		if (link) throw redirect(303, link);
		return { success: true };
	},

	readAll: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/notifications/read-all', { method: 'POST' });
		return response.ok ? { success: true } : fail(response.status);
	},

	preferences: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const response = await apiFetch(fetch, cookies, '/api/notifications/preferences', {
			method: 'PUT',
			body: JSON.stringify({ email_account: data.get('email_account') === 'true', email_promos: data.get('email_promos') === 'true' }),
		});
		return response.ok ? { saved: true } : fail(response.status);
	},
};
