import { m } from '$lib/paraglide/messages';
import type { Maintenance } from '$lib/admin/api';
import { utcInstant } from '$lib/admin/format';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * Maintenance (maintenance: view; changed with manage): whether everyone is held on the maintenance
 * page, what it says, and when Nomi is back, in UTC. The console itself stays open.
 */
export const load: PageServerLoad = async (event) => ({ maintenance: mustLoad(await getJson<Maintenance>(event, '/api/admin/maintenance')) });

const MAX_MESSAGE = 500;

export const actions: Actions = {
	save: async (event) => {
		const form = await event.request.formData();
		const message = field(form, 'message');
		const date = field(form, 'until_date');
		const time = field(form, 'until_time');
		if ([...message].length > MAX_MESSAGE) return refuse('save', m.admin_maintenance_message_long({ max: String(MAX_MESSAGE) }));
		const until = date ? utcInstant(date, time) : null;
		if ((date && !until) || (!date && time)) return refuse('save', m.admin_maintenance_until_invalid());
		return act<Maintenance>(event, 'save', '/api/admin/maintenance', 'PUT', { enabled: form.get('enabled') === 'on', message, until });
	},
};
