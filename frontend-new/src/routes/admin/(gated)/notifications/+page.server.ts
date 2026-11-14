import { m } from '$lib/paraglide/messages';
import { can } from '$lib/admin/access';
import type { AdminPlan, Broadcast } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * Notifications (notifications: view): what was sent to everyone's inbox, and (write) a new one, to
 * everyone or the people on one plan. The plans to choose from are there when the viewer may see
 * plans; without them it goes to everyone.
 */
export const load: PageServerLoad = async (event) => {
	const { access } = await event.parent();
	const [broadcasts, plans] = await Promise.all([
		getJson<Broadcast[]>(event, '/api/admin/notifications/broadcasts'),
		can(access, 'plans', 'view') ? getJson<AdminPlan[]>(event, '/api/admin/plans') : null,
	]);
	return { broadcasts: mustLoad(broadcasts), plans };
};

export const actions: Actions = {
	send: async (event) => {
		const form = await event.request.formData();
		const title = field(form, 'title');
		const body = field(form, 'body');
		const link = field(form, 'link');
		const audience = field(form, 'audience') || 'all';
		if (!title || !body) return refuse('send', m.admin_notify_required());
		// The backend takes only a page in the app (it opens in Nomi, never elsewhere).
		if (link && !/^\/(?!\/)/.test(link)) return refuse('send', m.admin_notify_link_invalid());
		return act<{ id: string; recipients: number }>(event, 'send', '/api/admin/notifications/broadcast', 'POST', { title, body, link: link || null, audience });
	},
};
