import { m } from '$lib/paraglide/messages';
import type { AppsOverview, ShipyardStatus, SyncedOrg, WebhookEndpoint } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * Integrations (integrations: view; write and manage change it): Shipyard, where projects are
 * published. Its connection and the webhook Nomi hears it through, catching up on missed events,
 * its apps (each tied to a project), the organizations synced to it, and the endpoints Nomi
 * forwards its events to. Each part loads on its own, so one Shipyard can't answer doesn't hide
 * the rest.
 */
export const load: PageServerLoad = async (event) => {
	const [status, apps, orgs, webhooks] = await Promise.all([
		getJson<ShipyardStatus>(event, '/api/admin/shipyard'),
		getJson<AppsOverview>(event, '/api/admin/shipyard/apps'),
		getJson<SyncedOrg[]>(event, '/api/admin/shipyard/orgs'),
		getJson<WebhookEndpoint[]>(event, '/api/admin/shipyard/webhooks'),
	]);
	return { status: mustLoad(status), apps, orgs, webhooks };
};

const id = (form: FormData) => field(form, 'id');

/** Events by type: exact (`publish.succeeded`), a prefix (`deployment.*`) or `*`; commas or spaces between. */
function readEvents(raw: string): string[] {
	return [...new Set(raw.split(/[\s,]+/).filter(Boolean))].sort();
}

export const actions: Actions = {
	registerWebhook: (event) => act<ShipyardStatus>(event, 'registerWebhook', '/api/admin/shipyard/webhook', 'POST'),

	catchUp: (event) => act<{ orgs: number; new_events: number; errors: string[] }>(event, 'catchUp', '/api/admin/shipyard/events/catch-up', 'POST'),

	link: async (event) => {
		const form = await event.request.formData();
		const orgId = field(form, 'org_id');
		const app = field(form, 'app');
		if (!orgId || !app) return refuse('link', m.admin_shipyard_app_required());
		return act(event, 'link', '/api/admin/shipyard/apps/link', 'POST', { org_id: orgId, app, project_id: field(form, 'project_id') || null });
	},

	syncOrg: async (event) => {
		const org = id(await event.request.formData());
		if (!org) return refuse('syncOrg', m.admin_shipyard_org_required());
		return act(event, 'syncOrg', `/api/admin/shipyard/orgs/${encodeURIComponent(org)}/sync`, 'POST');
	},

	removeOrg: async (event) => {
		const org = id(await event.request.formData());
		if (!org) return refuse('removeOrg', m.admin_shipyard_org_required());
		return act(event, 'removeOrg', `/api/admin/shipyard/orgs/${encodeURIComponent(org)}`, 'DELETE');
	},

	createEndpoint: async (event) => {
		const form = await event.request.formData();
		const url = field(form, 'url');
		const events = readEvents(field(form, 'events'));
		if (!url) return refuse('createEndpoint', m.admin_endpoint_url_required());
		if (events.length === 0) return refuse('createEndpoint', m.admin_endpoint_events_required());
		return act<{ id: string; secret: string | null; own: boolean }>(event, 'createEndpoint', '/api/admin/shipyard/webhooks', 'POST', { url, events, org: field(form, 'org') || null });
	},

	toggleEndpoint: async (event) => {
		const form = await event.request.formData();
		if (!id(form)) return refuse('toggleEndpoint', m.admin_endpoint_required());
		return act(event, 'toggleEndpoint', `/api/admin/shipyard/webhooks/${encodeURIComponent(id(form))}`, 'PATCH', { enabled: form.get('enabled') === 'on' });
	},

	testEndpoint: async (event) => {
		const endpoint = id(await event.request.formData());
		if (!endpoint) return refuse('testEndpoint', m.admin_endpoint_required());
		return act(event, 'testEndpoint', `/api/admin/shipyard/webhooks/${encodeURIComponent(endpoint)}/test`, 'POST');
	},

	rotateEndpoint: async (event) => {
		const endpoint = id(await event.request.formData());
		if (!endpoint) return refuse('rotateEndpoint', m.admin_endpoint_required());
		return act<{ secret: string | null }>(event, 'rotateEndpoint', `/api/admin/shipyard/webhooks/${encodeURIComponent(endpoint)}/rotate`, 'POST');
	},

	deleteEndpoint: async (event) => {
		const endpoint = id(await event.request.formData());
		if (!endpoint) return refuse('deleteEndpoint', m.admin_endpoint_required());
		return act(event, 'deleteEndpoint', `/api/admin/shipyard/webhooks/${encodeURIComponent(endpoint)}`, 'DELETE');
	},
};
