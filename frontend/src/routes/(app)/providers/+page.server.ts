import { connectProvider, disconnectProvider, loadCatalog, loadConnections, PERSONAL_CONNECTIONS } from '$lib/server/providers';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [providers, connections] = await Promise.all([loadCatalog(fetch, cookies), loadConnections(fetch, cookies, PERSONAL_CONNECTIONS)]);
	return { providers: providers ?? [], connections: connections ?? [], loadFailed: !providers || !connections };
};

export const actions: Actions = {
	connect: async ({ request, cookies, fetch }) => connectProvider(fetch, cookies, PERSONAL_CONNECTIONS, await request.formData()),
	disconnect: async ({ request, cookies, fetch }) => disconnectProvider(fetch, cookies, PERSONAL_CONNECTIONS, await request.formData()),
};
