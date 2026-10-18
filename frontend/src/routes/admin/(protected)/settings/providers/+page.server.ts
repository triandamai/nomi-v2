import { ADMIN_CONNECTIONS, connectProvider, disconnectProvider, loadCatalog, loadConnections } from '$lib/server/providers';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [providers, connections] = await Promise.all([loadCatalog(fetch, cookies), loadConnections(fetch, cookies, ADMIN_CONNECTIONS)]);
	return { providers: providers ?? [], connections: connections ?? [], loadFailed: !providers || !connections };
};

export const actions: Actions = {
	connect: async ({ request, cookies, fetch }) => connectProvider(fetch, cookies, ADMIN_CONNECTIONS, await request.formData()),
	disconnect: async ({ request, cookies, fetch }) => disconnectProvider(fetch, cookies, ADMIN_CONNECTIONS, await request.formData()),
};
