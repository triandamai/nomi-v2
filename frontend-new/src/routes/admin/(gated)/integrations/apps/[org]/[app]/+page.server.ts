import { redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { AppDetail, AppSecret } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * A Shipyard app (integrations: view): its environments with their services and last publish,
 * and its secrets. With write: duplicate an environment; with manage: delete one, set or delete a
 * secret, or delete the app.
 */
export const load: PageServerLoad = async (event) => {
	const base = appPath(event.params);
	const [app, secrets] = await Promise.all([getJson<AppDetail>(event, base), getJson<AppSecret[]>(event, `${base}/secrets`)]);
	return { title: event.params.app, app: mustLoad(app), secrets };
};

const appPath = (params: { org: string; app: string }) => `/api/admin/shipyard/orgs/${encodeURIComponent(params.org)}/apps/${encodeURIComponent(params.app)}`;
/** Environment names as Shipyard takes them. */
const ENV = /^[a-z0-9][a-z0-9-]{0,62}$/;
/** Secret names: capitals, digits and underscores. */
const SECRET = /^[A-Z_][A-Z0-9_]{0,127}$/;

export const actions: Actions = {
	duplicateEnv: async (event) => {
		const form = await event.request.formData();
		const from = field(form, 'from');
		const to = field(form, 'to');
		if (!ENV.test(from) || !ENV.test(to)) return refuse('duplicateEnv', m.admin_shipyard_env_invalid());
		return act(event, 'duplicateEnv', `${appPath(event.params)}/envs/${encodeURIComponent(from)}/duplicate`, 'POST', { to, copy_secrets: form.get('copy_secrets') === 'on' });
	},

	deleteEnv: async (event) => {
		const env = field(await event.request.formData(), 'env');
		if (!ENV.test(env)) return refuse('deleteEnv', m.admin_shipyard_env_invalid());
		return act(event, 'deleteEnv', `${appPath(event.params)}/envs/${encodeURIComponent(env)}`, 'DELETE');
	},

	/** A secret for every environment or one; left empty, Shipyard makes one up. */
	setSecret: async (event) => {
		const form = await event.request.formData();
		const name = field(form, 'name');
		const value = field(form, 'value');
		const environment = field(form, 'environment');
		if (!SECRET.test(name)) return refuse('setSecret', m.admin_shipyard_secret_invalid());
		return act(event, 'setSecret', `${appPath(event.params)}/secrets/${name}`, 'PUT', { ...(value && { value }), ...(environment && { environment }) });
	},

	deleteSecret: async (event) => {
		const form = await event.request.formData();
		const name = field(form, 'name');
		const environment = field(form, 'environment');
		if (!SECRET.test(name)) return refuse('deleteSecret', m.admin_shipyard_secret_invalid());
		return act(event, 'deleteSecret', `${appPath(event.params)}/secrets/${name}${environment ? `?environment=${encodeURIComponent(environment)}` : ''}`, 'DELETE');
	},

	deleteApp: async (event) => {
		const deleted = await act(event, 'deleteApp', appPath(event.params), 'DELETE');
		if ('ok' in deleted && deleted.ok) redirect(303, '/admin/integrations');
		return deleted;
	},
};
