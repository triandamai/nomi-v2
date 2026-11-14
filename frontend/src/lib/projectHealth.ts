// What a project card says about where a project stands: how much the crew is waiting on the
// person for, and whether it needs their attention (open issues, a failed publish, the live app down).
import type { ProjectSummary } from '$lib/types';

/** Questions and approvals waiting on the person in the project's chats. */
export function needsYou(project: ProjectSummary): number {
	return project.waiting_on_you ?? 0;
}

/** Its live app is down, or its latest publish failed. */
export function isBroken(project: ProjectSummary): boolean {
	return project.live_status === 'down' || project.last_deploy?.status === 'failed';
}

/** What the Needs attention tab lists. */
export function needsAttention(project: ProjectSummary): boolean {
	return project.open_issues > 0 || needsYou(project) > 0 || isBroken(project);
}
