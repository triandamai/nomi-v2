import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Plan } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export interface Broadcast {
	id: string;
	title: string;
	body: string;
	link: string | null;
	audience: string;
	recipients: number;
	read: number;
	created_at: string;
}

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [sentResponse, plansResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/admin/notifications/broadcasts'),
		apiFetch(fetch, cookies, '/api/admin/plans'),
	]);
	const sent: Broadcast[] = sentResponse.ok ? ((await sentResponse.json()) as Broadcast[]) : [];
	const plans: Plan[] = plansResponse.ok ? ((await plansResponse.json()) as Plan[]) : [];
	return { sent, plans };
};

export const actions: Actions = {
	send: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const text = (name: string) => {
			const value = data.get(name);
			return typeof value === 'string' ? value.trim() : '';
		};
		const title = text('title');
		const body = text('body');
		if (!title || !body) return fail(400, { error: m.broadcast_required() });
		const response = await apiFetch(fetch, cookies, '/api/admin/notifications/broadcast', {
			method: 'POST',
			body: JSON.stringify({ title, body, link: text('link') || null, audience: text('audience') || 'all' }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.broadcast_failed() });
		}
		const sent = (await response.json()) as { recipients: number };
		return { sent: sent.recipients };
	},
};
