import type { AdminSandbox } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

const UUID = /^[0-9a-f-]{36}$/i;

/** Every sandbox each project has had (projects: view); `?project=` shows one project's. */
export const load: PageServerLoad = async (event) => {
	const wanted = event.url.searchParams.get('project');
	const project = wanted && UUID.test(wanted) ? wanted : null;
	const loaded = mustLoad(await getJson<{ sandboxes: AdminSandbox[] }>(event, `/api/admin/sandboxes${project ? `?project=${project}` : ''}`));
	return { project, sandboxes: loaded.ok ? { ok: true as const, data: loaded.data?.sandboxes ?? [] } : loaded };
};
