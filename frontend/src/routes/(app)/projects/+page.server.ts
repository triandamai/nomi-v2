import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { OrgList, ProjectList } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [orgsResponse, projectsResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/orgs'),
		apiFetch(fetch, cookies, '/api/projects'),
	]);
	if (!orgsResponse.ok || !projectsResponse.ok) {
		throw error(500, m.err_load_projects());
	}
	const orgs = (await orgsResponse.json()) as OrgList;
	const list = (await projectsResponse.json()) as ProjectList;
	// No organization picked yet: the page asks where projects should live before anything else.
	return { orgs: orgs.orgs, orgId: list.org_id, projects: list.projects };
};

async function pickOrg(fetch: typeof globalThis.fetch, cookies: Parameters<typeof apiFetch>[1], orgId: string) {
	return apiFetch(fetch, cookies, '/api/preferences/project-org', {
		method: 'PUT',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ org_id: orgId }),
	});
}

export const actions: Actions = {
	// Picks the organization projects are listed from and started in.
	chooseOrg: async ({ request, cookies, fetch }) => {
		const orgId = (await request.formData()).get('org_id');
		if (typeof orgId !== 'string' || !orgId) {
			return fail(400, { orgError: m.org_err_pick() });
		}
		const response = await pickOrg(fetch, cookies, orgId);
		if (!response.ok) {
			return fail(response.status, { orgError: m.org_err_action() });
		}
		return { chosen: true };
	},

	// A new organization, owned by the person, which becomes where their projects go.
	createOrg: async ({ request, cookies, fetch }) => {
		const name = String((await request.formData()).get('name') ?? '').trim();
		if (!name) {
			return fail(400, { orgError: m.org_err_name() });
		}
		const response = await apiFetch(fetch, cookies, '/api/orgs', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name }),
		});
		if (!response.ok) {
			return fail(response.status, { orgError: m.org_err_create() });
		}
		const { id } = (await response.json()) as { id: string };
		await pickOrg(fetch, cookies, id);
		return { chosen: true };
	},

	// Creates the project row up front, in the same request as the chat session — the workspace
	// page (keyed by this session id) recognizes it as a project immediately, rather than
	// depending on the planning agent reliably calling its own create_project tool once the
	// conversation gets going (that dependency turned out to be unreliable in practice).
	createProject: async ({ request, cookies, fetch }) => {
		const orgId = (await request.formData()).get('org_id');
		const response = await apiFetch(fetch, cookies, '/api/projects', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(typeof orgId === 'string' && orgId ? { org_id: orgId } : {}),
		});
		// No organization picked yet (or it's gone): the page shows the picker.
		if (response.status === 409 || response.status === 404) {
			return fail(409, { needsOrg: true });
		}
		if (!response.ok) {
			throw error(500, m.err_new_project());
		}
		const { session_id } = (await response.json()) as { session_id: string; project_id: string };
		throw redirect(303, `/projects/session/${session_id}`);
	},

	// A new project to import existing code into: the page fills it, then opens it.
	createForImport: async ({ request, cookies, fetch }) => {
		const orgId = (await request.formData()).get('org_id');
		const response = await apiFetch(fetch, cookies, '/api/projects', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(typeof orgId === 'string' && orgId ? { org_id: orgId } : {}),
		});
		if (response.status === 409 || response.status === 404) return fail(409, { needsOrg: true });
		if (!response.ok) return fail(500, { importError: m.err_new_project() });
		const { session_id, project_id } = (await response.json()) as { session_id: string; project_id: string };
		return { imported: { session_id, project_id } };
	},
};
