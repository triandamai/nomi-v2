import { isRedirect } from '@sveltejs/kit';
import type { DeployOverview, ProjectDetail, ProjectList } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';

// A project's workspace on the server: the project (its files and chat), where it's live (the
// list carries the address; the project itself doesn't), and finding a project by its chat.

async function call(event: ApiEvent, path: string): Promise<Response> {
	try {
		return await apiFetch(event, path);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

export type LoadedProject = { project: ProjectDetail; live: { url: string; deploys: number } | null; domain: string | null } | { project: null; status: number };

export async function loadProject(event: ApiEvent, id: string): Promise<LoadedProject> {
	const [detail, list] = await Promise.all([call(event, `/api/projects/${encodeURIComponent(id)}`), call(event, '/api/projects')]);
	if (!detail.ok) return { project: null, status: detail.status };
	const projects = list.ok ? ((await list.json()) as ProjectList) : null;
	const summary = projects?.projects.find((one) => one.id === id);
	return {
		project: (await detail.json()) as ProjectDetail,
		live: summary?.deploy_url ? { url: summary.deploy_url, deploys: summary.deploys_count ?? 0 } : null,
		domain: projects?.base_domain ?? null,
	};
}

/** The project a chat belongs to, or null when it's an ordinary chat. */
export async function projectOfChat(event: ApiEvent, sessionId: string): Promise<string | null> {
	const response = await call(event, `/api/projects/by-session/${encodeURIComponent(sessionId)}`);
	return response.ok ? ((await response.json()) as { id: string }).id : null;
}

/** What's published and what's waiting; null where the backend can't say. */
export async function loadDeploy(event: ApiEvent, id: string): Promise<DeployOverview | null> {
	const response = await call(event, `/api/projects/${encodeURIComponent(id)}/deploy`);
	return response.ok ? ((await response.json()) as DeployOverview) : null;
}

export type Sent = { ok: true } | { ok: false; status: number; message: string };

async function post(event: ApiEvent, path: string, body?: unknown): Promise<Sent> {
	let response: Response;
	try {
		response = await apiFetch(event, path, { method: 'POST', ...(body !== undefined && { body: JSON.stringify(body) }) });
	} catch (error) {
		if (isRedirect(error)) throw error;
		return { ok: false, status: 503, message: '' };
	}
	return response.ok ? { ok: true } : { ok: false, status: response.status, message: (await response.text()).trim().slice(0, 300) };
}

/** Sends the project's current files live (production); the publish runs on its own. */
export const publish = (event: ApiEvent, id: string) => post(event, `/api/projects/${encodeURIComponent(id)}/deploy/publish`, {});
/** Publishes an earlier deploy's build again, as a new deploy. */
export const rollBack = (event: ApiEvent, id: string, publishId: string) =>
	post(event, `/api/projects/${encodeURIComponent(id)}/deploy/publishes/${encodeURIComponent(publishId)}/rollback`);
