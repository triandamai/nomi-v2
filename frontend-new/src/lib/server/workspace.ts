import { isRedirect } from '@sveltejs/kit';
import type { ProjectDetail, ProjectList } from '$lib/adapters/api';
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
