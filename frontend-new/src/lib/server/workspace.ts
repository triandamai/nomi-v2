import { isRedirect } from '@sveltejs/kit';
import type { ContainerUsage, DatabaseTable, DeployOverview, ExportSummary, ProjectShowcase, DomainAnswer, MetricsRange, ProjectDetail, ProjectDomain, ProjectList, ProjectMembers, ProjectMetrics, ProjectSecrets, QueryResult, SandboxLogs, SandboxStatus } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';
import { readTable } from '$lib/projects/database';

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

export type DatabaseView = { tables: DatabaseTable[]; table: DatabaseTable | null; rows: QueryResult | null } | { error: string; status: number };

/**
 * A project's tables (development or production) and the first rows of the one asked for (else the
 * first), read only: the statement is built here from a table the backend listed.
 */
export async function readDatabase(event: ApiEvent, id: string, source: 'development' | 'production', wanted: string | null, limit = 50): Promise<DatabaseView> {
	const at = `/api/projects/${encodeURIComponent(id)}/database`;
	const listed = await call(event, `${at}/tables?source=${source}`);
	if (!listed.ok) return { error: (await listed.text()).trim().slice(0, 300), status: listed.status };
	const tables = (await listed.json()) as DatabaseTable[];
	const table = tables.find((one) => `${one.schema}.${one.name}` === wanted) ?? tables[0] ?? null;
	if (!table) return { tables, table: null, rows: null };
	let response: Response;
	try {
		response = await apiFetch(event, `${at}/query`, { method: 'POST', body: JSON.stringify({ sql: readTable(table, limit), source }) });
	} catch (error) {
		if (isRedirect(error)) throw error;
		return { error: '', status: 503 };
	}
	if (!response.ok) return { error: (await response.text()).trim().slice(0, 300), status: response.status };
	return { tables, table, rows: (await response.json()) as QueryResult };
}

/** A workspace a project can move to (`/transfer/orgs?q=`); never the one it's in, nor a personal one. */
export type TransferTarget = { id: string; name: string; is_personal: boolean; owner_name: string | null; my_role: string | null };

export async function transferTargets(event: ApiEvent, id: string, query: string): Promise<TransferTarget[] | null> {
	const response = await call(event, `/api/projects/${encodeURIComponent(id)}/transfer/orgs${query ? `?q=${encodeURIComponent(query)}` : ''}`);
	return response.ok ? ((await response.json()) as TransferTarget[]) : null;
}

async function send(event: ApiEvent, path: string, method: string, body?: unknown): Promise<Sent> {
	let response: Response;
	try {
		response = await apiFetch(event, path, { method, ...(body !== undefined && { body: JSON.stringify(body) }) });
	} catch (error) {
		if (isRedirect(error)) throw error;
		return { ok: false, status: 503, message: '' };
	}
	return response.ok ? { ok: true } : { ok: false, status: response.status, message: (await response.text()).trim().slice(0, 300) };
}

/** Gives the project to another workspace; the person types its name to confirm. */
export const transfer = (event: ApiEvent, id: string, targetOrgId: string, confirmName: string) =>
	send(event, `/api/projects/${encodeURIComponent(id)}/transfer`, 'POST', { target_org_id: targetOrgId, confirm_name: confirmName });
/** Stops serving the live app (production); code, data and deploys stay, and publishing brings it back. */
export const takeDown = (event: ApiEvent, id: string) => send(event, `/api/projects/${encodeURIComponent(id)}/deploy/environments/production`, 'DELETE');

const at = (id: string) => `/api/projects/${encodeURIComponent(id)}`;

/** Who's in the project's workspace and their place on the project; null where it can't be read. */
export async function loadPeople(event: ApiEvent, id: string): Promise<ProjectMembers | null> {
	const response = await call(event, `${at(id)}/members`);
	return response.ok ? ((await response.json()) as ProjectMembers) : null;
}

/** Invites an address as `build` (admin), `prompt` (member) or `view` (viewer). */
export const invitePerson = (event: ApiEvent, id: string, email: string, role: 'build' | 'prompt' | 'view') => send(event, `${at(id)}/invites`, 'POST', { email, role });
export const setPersonRole = (event: ApiEvent, id: string, userId: string, role: 'admin' | 'member' | 'viewer') => send(event, `${at(id)}/members/${encodeURIComponent(userId)}`, 'PUT', { role });
export const removePerson = (event: ApiEvent, id: string, userId: string) => send(event, `${at(id)}/members/${encodeURIComponent(userId)}`, 'DELETE');
/** Opens the project to the whole workspace, or only to the people added. */
export const setAccess = (event: ApiEvent, id: string, access: 'org' | 'members') => send(event, `${at(id)}/access`, 'PUT', { access });

/** The project's environment variables, names only. */
export async function loadSecrets(event: ApiEvent, id: string): Promise<ProjectSecrets | null> {
	const response = await call(event, `${at(id)}/secrets`);
	return response.ok ? ((await response.json()) as ProjectSecrets) : null;
}
export const setSecret = (event: ApiEvent, id: string, name: string, value: string) => send(event, `${at(id)}/secrets/${encodeURIComponent(name)}`, 'PUT', { value });
export const removeSecret = (event: ApiEvent, id: string, name: string) => send(event, `${at(id)}/secrets/${encodeURIComponent(name)}`, 'DELETE');

/** What an export holds; null where it can't be read. */
export async function exportSummary(event: ApiEvent, id: string): Promise<ExportSummary | null> {
	const response = await call(event, `${at(id)}/export`);
	return response.ok ? ((await response.json()) as ExportSummary) : null;
}

/** One commit of the project's files to `owner/name` on GitHub, with the person's GitHub. */
export const pushToGithub = (event: ApiEvent, id: string, push: { repo: string; create: boolean; private: boolean }) => send(event, `${at(id)}/export/github`, 'POST', push);

/** Whether the live app is on show; null where it can't be read. */
export async function loadShowcase(event: ApiEvent, id: string): Promise<ProjectShowcase | null> {
	const response = await call(event, `${at(id)}/showcase`);
	return response.ok ? ((await response.json()) as ProjectShowcase) : null;
}
export const setShowcase = (event: ApiEvent, id: string, showcase: boolean, blurb: string | null) => send(event, `${at(id)}/showcase`, 'PUT', { showcase, blurb });

export type ProjectFile = { path: string; content: string; binary: false } | { path: string; binary: true } | { error: string; status: number };

/** One of the project's files as text; an image, a font or anything not UTF-8 is said to be binary. */
export async function projectFile(event: ApiEvent, id: string, path: string): Promise<ProjectFile> {
	const response = await call(event, `${at(id)}/files/${path.split('/').map(encodeURIComponent).join('/')}`);
	if (!response.ok) return { error: (await response.text()).trim().slice(0, 200), status: response.status };
	const bytes = new Uint8Array(await response.arrayBuffer());
	try {
		return { path, content: new TextDecoder('utf-8', { fatal: true }).decode(bytes), binary: false };
	} catch {
		return { path, binary: true };
	}
}

/** The project's sandbox: where it runs, whether the sandbox may be used, its phase; null where it can't be read. */
export async function loadSandbox(event: ApiEvent, id: string): Promise<SandboxStatus | null> {
	const response = await call(event, `${at(id)}/sandbox`);
	return response.ok ? ((await response.json()) as SandboxStatus) : null;
}

/** The sandbox's dev server output from `offset` on; null where it can't be read. */
export async function sandboxLogs(event: ApiEvent, id: string, offset: number): Promise<SandboxLogs | null> {
	const response = await call(event, `${at(id)}/sandbox/logs?offset=${offset}`);
	return response.ok ? ((await response.json()) as SandboxLogs) : null;
}

/**
 * The page's heartbeat, which keeps the sandbox awake while the workspace is open. `runtime=sandbox`
 * tells the backend this is the sandbox page, so it leaves Koda's commands to the sandbox.
 */
export async function sandboxBeat(event: ApiEvent, id: string): Promise<boolean> {
	return (await call(event, `${at(id)}/runtime?runtime=sandbox`)).ok;
}

export const startSandbox = (event: ApiEvent, id: string) => post(event, `${at(id)}/sandbox/start`);
export const stopSandbox = (event: ApiEvent, id: string) => post(event, `${at(id)}/sandbox/stop`);

/** Whether the project keeps any tables (its development database); false where that can't be read. */
export async function hasTables(event: ApiEvent, id: string): Promise<boolean> {
	const response = await call(event, `${at(id)}/database/tables?source=development`);
	return response.ok && ((await response.json()) as DatabaseTable[]).length > 0;
}
