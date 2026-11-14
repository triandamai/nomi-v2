import { isRedirect } from '@sveltejs/kit';
import type { ContainerUsage, DeployOverview, DomainAnswer, MetricsRange, ProjectDetail, ProjectDomain, ProjectList, ProjectMetrics } from '$lib/adapters/api';
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

/** The app's address; null where the backend can't say. */
export async function loadDomain(event: ApiEvent, id: string): Promise<ProjectDomain | null> {
	const response = await call(event, `/api/projects/${encodeURIComponent(id)}/domain`);
	return response.ok ? ((await response.json()) as ProjectDomain) : null;
}

/** Whether a name is free, as the hosting answers it (its reason and free names when it isn't). */
export async function checkDomain(event: ApiEvent, id: string, name: string): Promise<DomainAnswer | { error: string }> {
	const response = await call(event, `/api/projects/${encodeURIComponent(id)}/domain/check?subdomain=${encodeURIComponent(name)}`);
	if (response.ok) return (await response.json()) as DomainAnswer;
	return { error: (await response.text()).trim().slice(0, 300) };
}

export type DomainChange = { ok: true; url: string | null } | { ok: false; status: number; answer?: DomainAnswer; message: string };

/** Moves production to another name; a refusal comes back with why and free names. */
export async function changeDomain(event: ApiEvent, id: string, name: string): Promise<DomainChange> {
	let response: Response;
	try {
		response = await apiFetch(event, `/api/projects/${encodeURIComponent(id)}/domain`, { method: 'PUT', body: JSON.stringify({ subdomain: name }) });
	} catch (error) {
		if (isRedirect(error)) throw error;
		return { ok: false, status: 503, message: '' };
	}
	const text = await response.text();
	if (response.ok) return { ok: true, url: ((text ? JSON.parse(text) : {}) as { url?: string }).url ?? null };
	if (response.status === 409) {
		try {
			const answer = JSON.parse(text) as DomainAnswer;
			return { ok: false, status: 409, answer, message: answer.message ?? '' };
		} catch {
			return { ok: false, status: 409, message: text.trim().slice(0, 300) };
		}
	}
	return { ok: false, status: response.status, message: text.trim().slice(0, 300) };
}

export type Monitoring = { metrics: ProjectMetrics | null; usage: ContainerUsage[] | null; logs: string[] | null; note: string | null };

/** The running app's numbers, what it uses and its last lines; each null where it couldn't be had. */
export async function loadMonitor(event: ApiEvent, id: string, range: MetricsRange): Promise<Monitoring> {
	const at = `/api/projects/${encodeURIComponent(id)}/deploy`;
	const [metrics, usage, logs] = await Promise.all([call(event, `${at}/metrics?range=${range}`), call(event, `${at}/usage`), call(event, `${at}/logs?tail=40`)]);
	const logBody = logs.ok ? ((await logs.json()) as { lines: string[]; note?: string }) : null;
	return {
		metrics: metrics.ok ? ((await metrics.json()) as ProjectMetrics) : null,
		usage: usage.ok ? ((await usage.json()) as { containers: ContainerUsage[] }).containers : null,
		logs: logBody?.lines ?? null,
		note: logBody?.note ?? null,
	};
}
