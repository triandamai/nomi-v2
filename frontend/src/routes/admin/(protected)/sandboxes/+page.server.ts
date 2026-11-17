import { apiFetch } from '$lib/server/api';
import type { AdminSandbox } from '$lib/adminProjects';
import type { PageServerLoad } from './$types';

const UUID = /^[0-9a-f-]{36}$/i;

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const wanted = url.searchParams.get('project');
	const project = wanted && UUID.test(wanted) ? wanted : null;
	const response = await apiFetch(fetch, cookies, `/api/admin/sandboxes${project ? `?project=${project}` : ''}`);
	return { project, sandboxes: response.ok ? ((await response.json()) as { sandboxes: AdminSandbox[] }).sandboxes : null };
};
