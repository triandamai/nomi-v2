import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { IssueKind, IssueList, IssueSeverity, ProjectDetail } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';
import { ISSUE_KINDS, ISSUE_SEVERITIES, issuesQuery, readIssueFilters } from '$lib/issues';

export const load: PageServerLoad = async ({ cookies, fetch, params, url }) => {
	const filters = readIssueFilters(url.searchParams);
	const [projectResponse, issuesResponse] = await Promise.all([
		apiFetch(fetch, cookies, `/api/projects/${params.projectId}`),
		apiFetch(fetch, cookies, `/api/projects/${params.projectId}/issues?${issuesQuery(filters)}`),
	]);
	if (projectResponse.status === 404) throw error(404, m.err_load_projects());
	if (!projectResponse.ok || !issuesResponse.ok) throw error(500, m.err_load_projects());
	const project = (await projectResponse.json()) as ProjectDetail;
	const list = (await issuesResponse.json()) as IssueList;
	return {
		project: { id: project.id, name: project.name, sessionId: project.session_id, orgName: project.org_name },
		issues: list.issues,
		counts: list.counts,
		filters,
	};
};

type Fields = { title: string; body: string; kind: IssueKind; severity: IssueSeverity };

function readFields(data: FormData): Fields | null {
	const kind = String(data.get('kind') ?? '');
	const severity = String(data.get('severity') ?? '');
	if (!ISSUE_KINDS.includes(kind as IssueKind) || !ISSUE_SEVERITIES.includes(severity as IssueSeverity)) return null;
	return {
		title: String(data.get('title') ?? '').trim(),
		body: String(data.get('body') ?? ''),
		kind: kind as IssueKind,
		severity: severity as IssueSeverity,
	};
}

function send(fetch: typeof globalThis.fetch, cookies: Parameters<typeof apiFetch>[1], path: string, method: string, body?: unknown) {
	return apiFetch(fetch, cookies, path, {
		method,
		headers: { 'Content-Type': 'application/json' },
		body: body === undefined ? undefined : JSON.stringify(body),
	});
}

export const actions: Actions = {
	create: async ({ request, cookies, fetch, params }) => {
		const fields = readFields(await request.formData());
		if (!fields) return fail(400, { error: m.issues_err_save() });
		if (!fields.title) return fail(400, { error: m.issues_err_title(), fields });
		const response = await send(fetch, cookies, `/api/projects/${params.projectId}/issues`, 'POST', fields);
		if (!response.ok) return fail(response.status, { error: m.issues_err_save(), fields });
		const issue = (await response.json()) as { number: number };
		return { saved: issue.number };
	},

	update: async ({ request, cookies, fetch, params }) => {
		const data = await request.formData();
		const number = Number(data.get('number'));
		const fields = readFields(data);
		if (!fields || !Number.isInteger(number)) return fail(400, { error: m.issues_err_save() });
		if (!fields.title) return fail(400, { error: m.issues_err_title(), fields });
		const response = await send(fetch, cookies, `/api/projects/${params.projectId}/issues/${number}`, 'PATCH', fields);
		if (!response.ok) return fail(response.status, { error: m.issues_err_save(), fields });
		return { saved: number };
	},

	status: async ({ request, cookies, fetch, params }) => {
		const data = await request.formData();
		const number = Number(data.get('number'));
		const status = String(data.get('status'));
		if (!Number.isInteger(number) || (status !== 'open' && status !== 'closed')) return fail(400, { error: m.issues_err_save() });
		const response = await send(fetch, cookies, `/api/projects/${params.projectId}/issues/${number}`, 'PATCH', { status });
		if (!response.ok) return fail(response.status, { error: m.issues_err_save() });
		return { saved: number };
	},

	delete: async ({ request, cookies, fetch, params }) => {
		const number = Number((await request.formData()).get('number'));
		const response = await send(fetch, cookies, `/api/projects/${params.projectId}/issues/${number}`, 'DELETE');
		if (!response.ok) return fail(response.status, { error: m.issues_err_save() });
		return { deleted: number };
	},

	// Hands the issue to the crew in the project's chat, and opens it there.
	askKoda: async ({ request, cookies, fetch, params }) => {
		const number = Number((await request.formData()).get('number'));
		const [projectResponse, issueResponse] = await Promise.all([
			apiFetch(fetch, cookies, `/api/projects/${params.projectId}`),
			apiFetch(fetch, cookies, `/api/projects/${params.projectId}/issues/${number}`),
		]);
		if (!projectResponse.ok || !issueResponse.ok) return fail(404, { error: m.issues_err_save() });
		const project = (await projectResponse.json()) as ProjectDetail;
		const issue = (await issueResponse.json()) as Fields & { number: number };
		const kind = { feature: m.issues_kind_feature, enhancement: m.issues_kind_enhancement, bug: m.issues_kind_bug }[issue.kind]();
		const severity = {
			low: m.issues_severity_low,
			medium: m.issues_severity_medium,
			high: m.issues_severity_high,
			critical: m.issues_severity_critical,
		}[issue.severity]();
		const text = [
			m.issues_koda_message({ number: issue.number, kind: kind.toLowerCase(), severity: severity.toLowerCase(), title: issue.title }),
			issue.body.trim(),
		]
			.filter(Boolean)
			.join('\n\n');
		const response = await send(fetch, cookies, `/api/sessions/${project.session_id}/messages`, 'POST', { text });
		if (!response.ok) return fail(response.status, { error: m.issues_err_save() });
		throw redirect(303, `/projects/session/${project.session_id}`);
	},
};
