import { isRedirect } from '@sveltejs/kit';
import type { OrgDetail, OrgInvite, OrgList, OrgRole, OrgSummary } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';

// Settings › Workspace's calls to the backend, as the person: the workspace with its members (and,
// for owners and admins, its open invites), renaming it, inviting an address, taking an invite
// back, changing someone's role, and removing someone or leaving. The backend's reasons are plain
// words ("they're already a member"), passed on as they are.

export type Sent<T> = { ok: true; data: T } | { ok: false; status: number; message: string };

async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

async function send<T>(event: ApiEvent, path: string, method: string, body?: unknown): Promise<Sent<T>> {
	const response = await call(event, path, { method, ...(body !== undefined && { body: JSON.stringify(body) }) });
	const text = await response.text();
	if (!response.ok) return { ok: false, status: response.status, message: text.trim().slice(0, 200) };
	return { ok: true, data: (text ? JSON.parse(text) : null) as T };
}

export type WorkspaceData = { detail: OrgDetail; summary: OrgSummary | null; projectsHere: boolean } | { detail: null; status: number };

export async function loadWorkspace(event: ApiEvent, id: string): Promise<WorkspaceData> {
	const at = `/api/orgs/${encodeURIComponent(id)}`;
	const [detail, list] = await Promise.all([call(event, at), call(event, '/api/orgs')]);
	if (!detail.ok) return { detail: null, status: detail.status };
	const orgs = list.ok ? ((await list.json()) as OrgList) : null;
	return { detail: (await detail.json()) as OrgDetail, summary: orgs?.orgs.find((org) => org.id === id) ?? null, projectsHere: orgs?.project_org_id === id };
}

const org = (id: string) => `/api/orgs/${encodeURIComponent(id)}`;

export const rename = (event: ApiEvent, id: string, name: string) => send<unknown>(event, org(id), 'PATCH', { name });
export const invite = (event: ApiEvent, id: string, email: string, role: Exclude<OrgRole, 'owner'>) => send<OrgInvite>(event, `${org(id)}/invites`, 'POST', { email, role });
export const revoke = (event: ApiEvent, id: string, code: string) => send<void>(event, `${org(id)}/invites/${encodeURIComponent(code)}`, 'DELETE');
export const setRole = (event: ApiEvent, id: string, userId: string, role: OrgRole) => send<void>(event, `${org(id)}/members/${encodeURIComponent(userId)}`, 'PATCH', { role });
/** Removes someone, or leaves when it's the person themselves. */
export const removeMember = (event: ApiEvent, id: string, userId: string) => send<void>(event, `${org(id)}/members/${encodeURIComponent(userId)}`, 'DELETE');
