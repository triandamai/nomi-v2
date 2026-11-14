import { isRedirect } from '@sveltejs/kit';
import type { OrgMember, ProjectList } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';

// The Projects page's calls to the backend, as the person: the projects of the workspace they
// come from and who's in it, starting a project (its row and chat at once, then the description as
// its first message, as the old app does), and deleting one. Logic from the old app
// (frontend/src/routes/(app)/projects/+page.server.ts).

async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

export async function loadProjects(event: ApiEvent): Promise<{ list: ProjectList | null; members: OrgMember[] }> {
	const response = await call(event, '/api/projects');
	if (!response.ok) return { list: null, members: [] };
	const list = (await response.json()) as ProjectList;
	if (!list.org_id) return { list, members: [] };
	const org = await call(event, `/api/orgs/${encodeURIComponent(list.org_id)}`);
	const members = org.ok ? (((await org.json()) as { members?: OrgMember[] }).members ?? []) : [];
	return { list, members };
}

export type Started = { ok: true; sessionId: string; projectId: string; sent: boolean } | { ok: false; status: number; needsWorkspace?: boolean; message: string };

/** A new project in the workspace, its chat started with what the person described. */
export async function startProject(event: ApiEvent, orgId: string | null, description: string): Promise<Started> {
	const made = await call(event, '/api/projects', { method: 'POST', body: JSON.stringify(orgId ? { org_id: orgId } : {}) });
	if (!made.ok) {
		const message = (await made.text()).trim().slice(0, 200);
		return { ok: false, status: made.status, ...((made.status === 409 || made.status === 404) && { needsWorkspace: true }), message };
	}
	const { session_id: sessionId, project_id: projectId } = (await made.json()) as { session_id: string; project_id: string };
	const text = description.trim();
	const sent = text ? (await call(event, `/api/sessions/${sessionId}/messages`, { method: 'POST', body: JSON.stringify({ text }) })).ok : true;
	return { ok: true, sessionId, projectId, sent };
}

export async function deleteProject(event: ApiEvent, id: string): Promise<{ ok: true } | { ok: false; status: number; message: string }> {
	const response = await call(event, `/api/projects/${encodeURIComponent(id)}`, { method: 'DELETE' });
	return response.ok ? { ok: true } : { ok: false, status: response.status, message: (await response.text()).trim().slice(0, 200) };
}
