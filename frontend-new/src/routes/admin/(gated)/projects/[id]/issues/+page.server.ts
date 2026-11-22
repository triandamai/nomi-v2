import { m } from '$lib/paraglide/messages';
import type { AdminProjectIssues } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

/** One project's issues (projects: view), read only: the people work on them in the project. */
export const load: PageServerLoad = async (event) => {
	const issues = mustLoad(await getJson<AdminProjectIssues>(event, `/api/admin/projects/${encodeURIComponent(event.params.id)}/issues`));
	return { title: issues.ok ? m.admin_project_issues_title({ project: issues.data.project.name }) : m.admin_page_projects(), issues };
};
