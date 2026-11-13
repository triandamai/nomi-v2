import { m } from '$lib/paraglide/messages';
import type { Delivery, WebhookEndpoint } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/** An endpoint's deliveries (integrations: view), `?status=` narrowing them; write sends one again. */
export const load: PageServerLoad = async (event) => {
	const status = event.url.searchParams.get('status') ?? '';
	const query = /^[a-z_]{1,20}$/.test(status) ? `?status=${status}` : '';
	const [deliveries, endpoints] = await Promise.all([
		getJson<Delivery[]>(event, `/api/admin/shipyard/webhooks/${encodeURIComponent(event.params.id)}/deliveries${query}`),
		getJson<WebhookEndpoint[]>(event, '/api/admin/shipyard/webhooks'),
	]);
	const endpoint = endpoints.ok ? (endpoints.data.find((one) => one.id === event.params.id) ?? null) : null;
	return { title: endpoint?.url ?? m.admin_endpoint_deliveries(), status: query ? status : '', endpoint, deliveries: mustLoad(deliveries) };
};

export const actions: Actions = {
	redeliver: async (event) => {
		const delivery = field(await event.request.formData(), 'id');
		if (!delivery) return refuse('redeliver', m.admin_delivery_required());
		return act(event, 'redeliver', `/api/admin/shipyard/deliveries/${encodeURIComponent(delivery)}/redeliver`, 'POST');
	},
};
