import { isRedirect } from '@sveltejs/kit';
import type { DeployOverview, ExportSummary, OrgList, OrgMember, OrgRole, ProjectDetail, ProjectList, ProjectMembers, SessionSummary } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';

// The Projects page's calls to the backend, as the person: the projects of the workspace they
// come from and who's in it, which of their chats Koda is at work in, starting a project (its row
// and chat at once, then the description as its first message, as the old app does), deleting
// one, what the detail panel says about one, and the workspaces one can move to. Logic from the
// old app (frontend/src/routes/(app)/projects/+page.server.ts).

async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

/** A chat of a project's: its own (where Koda builds it) or one moved into it; `active` while a crew member works in it. */
export type ProjectChatRow = { id: string; title: string | null; updated_at: string; project: string; own: boolean; active: boolean };

export type LoadedProjects = {
	list: ProjectList | null;
	/** The status the list failed with, when it did. */
	status?: number;
	members: OrgMember[];
	chats: ProjectChatRow[];
	/** Whether this Nomi can publish; null where nothing here could be published anyway. */
	publishing: boolean | null;
};

/** The projects' chats, from the person's chats (a project's own chat and those moved into it). */
function chatsOf(sessions: SessionSummary[], ids: Set<string>): ProjectChatRow[] {
	return sessions.flatMap((session) => {
		const project = session.project_id && ids.has(session.project_id) ? session.project_id : session.in_project_id && ids.has(session.in_project_id) ? session.in_project_id : null;
		return project ? [{ id: session.id, title: session.title, updated_at: session.updated_at, project, own: session.project_id === project, active: Boolean(session.agent_active) }] : [];
	});
}

export async function loadProjects(event: ApiEvent): Promise<LoadedProjects> {
	const response = await call(event, '/api/projects');
	if (!response.ok) return { list: null, status: response.status, members: [], chats: [], publishing: null };
	const list = (await response.json()) as ProjectList;
	if (!list.org_id) return { list, members: [], chats: [], publishing: null };
	// Whether publishing is set up is the same for every project: asked of one that could be published.
	const candidate = list.projects.find((project) => !project.deploy_url && (project.files_count ?? 0) > 0);
	const [org, sessions, deploy] = await Promise.all([
		call(event, `/api/orgs/${encodeURIComponent(list.org_id)}`),
		list.projects.length ? call(event, '/api/sessions') : null,
		candidate ? call(event, `/api/projects/${encodeURIComponent(candidate.id)}/deploy`) : null,
	]);
	const members = org.ok ? (((await org.json()) as { members?: OrgMember[] }).members ?? []) : [];
	const all = sessions?.ok ? (((await sessions.json().catch(() => null)) as { sessions?: SessionSummary[] } | null)?.sessions ?? []) : [];
	const publishing = deploy ? (deploy.ok ? ((await deploy.json()) as DeployOverview).available : null) : null;
	return { list, members, chats: chatsOf(all, new Set(list.projects.map((project) => project.id))), publishing };
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

const at = (id: string) => `/api/projects/${encodeURIComponent(id)}`;

export type Sent = { ok: true } | { ok: false; status: number; message: string };

async function send(event: ApiEvent, path: string, body: unknown): Promise<Sent> {
	const response = await call(event, path, { method: 'POST', body: JSON.stringify(body) });
	return response.ok ? { ok: true } : { ok: false, status: response.status, message: (await response.text()).trim().slice(0, 200) };
}

/** A new name for a project (its builders only). */
export async function renameProject(event: ApiEvent, id: string, name: string): Promise<Sent> {
	const response = await call(event, at(id), { method: 'PATCH', body: JSON.stringify({ name }) });
	return response.ok ? { ok: true } : { ok: false, status: response.status, message: (await response.text()).trim().slice(0, 200) };
}

/** A copy of a project's files, stack, description and plan in the same workspace (not its publishing, data or people). */
export async function duplicateProject(event: ApiEvent, id: string): Promise<{ ok: true; projectId: string } | { ok: false; status: number; message: string }> {
	const response = await call(event, `${at(id)}/duplicate`, { method: 'POST', body: JSON.stringify({}) });
	if (!response.ok) return { ok: false, status: response.status, message: (await response.text()).trim().slice(0, 200) };
	return { ok: true, projectId: ((await response.json()) as { project_id: string }).project_id };
}

/** Says something to Koda in a project's own chat, where it builds the project. */
export const tellKoda = (event: ApiEvent, sessionId: string, text: string) => send(event, `/api/sessions/${encodeURIComponent(sessionId)}/messages`, { text });

/** What the detail panel says about a project; each part null where it couldn't be read. */
export type ProjectAbout = {
	files: number | null;
	stack: string | null;
	/** The database its files use (`postgres`, `sqlite` or `none`), as the export reads it. */
	database: string | null;
	people: { name: string; owner: boolean; me: boolean }[] | null;
	/** `failedNumber`: the latest publish's number when it failed. */
	deploys: { count: number; builtIn: number | null; liveNumber: number | null; liveAt: string | null; failedNumber: number | null } | null;
};

const seconds = (from: string, to: string | null) => (to ? Math.max(0, Math.round((Date.parse(to) - Date.parse(from)) / 1000)) : null);

export async function projectAbout(event: ApiEvent, id: string): Promise<ProjectAbout | null> {
	// The database is read from the files (as an export does), never by opening it: listing its
	// tables would set up the project's development database just to show the panel.
	const [detail, summary, members, deploy] = await Promise.all([
		call(event, at(id)),
		call(event, `${at(id)}/export`),
		call(event, `${at(id)}/members`),
		call(event, `${at(id)}/deploy`),
	]);
	if (!detail.ok) return null;
	const project = (await detail.json()) as ProjectDetail;
	// An answer without the list it should hold leaves that part out, rather than the whole panel.
	const team = members.ok ? ((await members.json().catch(() => null)) as ProjectMembers | null) : null;
	const overview = deploy.ok ? ((await deploy.json().catch(() => null)) as DeployOverview | null) : null;
	const production = overview?.publishes?.filter((one) => one.environment === 'production') ?? [];
	const built = production.find((one) => one.status === 'succeeded');
	const live = production.find((one) => one.is_live);
	return {
		files: project.files.length,
		stack: project.stack || null,
		database: summary.ok ? ((await summary.json()) as ExportSummary).database : null,
		// Who can open it: everyone in the workspace, or only the people added (and its owner).
		people: team?.people
			? team.people.filter((person) => team.access === 'org' || person.is_owner || person.project_role !== null).map((person) => ({ name: person.name, owner: person.is_owner, me: person.user_id === team.me }))
			: null,
		deploys: overview?.publishes
			? { count: production.filter((one) => one.status === 'succeeded').length, builtIn: built ? seconds(built.created_at, built.finished_at) : null, liveNumber: live?.version ?? null, liveAt: live?.finished_at ?? live?.created_at ?? null, failedNumber: production[0]?.status === 'failed' ? production[0].version : null }
			: null,
	};
}

/** A workspace a project can move to, with what the Move dialog says of it. */
export type MoveChoice = { id: string; name: string; members: number | null; role: OrgRole | null; owner: string | null };
export type MoveChoices = { current: { id: string; name: string; members: number } | null; targets: MoveChoice[] };

/** The workspace a project is in and those it can move to (never a personal one, as the backend says). */
export async function moveChoices(event: ApiEvent, id: string): Promise<MoveChoices | null> {
	const [targets, orgs, detail] = await Promise.all([call(event, `${at(id)}/transfer/orgs`), call(event, '/api/orgs'), call(event, at(id))]);
	if (!targets.ok) return null;
	const found = (await targets.json()) as { id: string; name: string; owner_name: string | null; my_role: string | null }[];
	const mine = orgs.ok ? ((await orgs.json()) as OrgList).orgs : [];
	const project = detail.ok ? ((await detail.json()) as ProjectDetail) : null;
	const here = project ? mine.find((org) => org.id === project.org_id) : undefined;
	const role = (value: string | null | undefined): OrgRole | null => (value === 'owner' || value === 'admin' || value === 'member' ? value : null);
	return {
		current: project ? { id: project.org_id, name: project.org_name, members: here?.members ?? 0 } : null,
		targets: found.map((target) => {
			const org = mine.find((one) => one.id === target.id);
			return { id: target.id, name: target.name, members: org?.members ?? null, role: role(org?.role ?? target.my_role), owner: target.owner_name };
		}),
	};
}
