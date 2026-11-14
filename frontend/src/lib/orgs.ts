// Workspaces: what a card and a page say about who's around (the newest thing that happened in
// its projects, when each member was last active), and why one couldn't be deleted.
import { m } from '$lib/paraglide/messages';
import { timeAgo } from '$lib/i18n';
import type { OrgActivity } from '$lib/types';

/** Someone seen this recently counts as active now. */
export const ACTIVE_NOW_MINUTES = 5;
const MINUTE_MS = 60_000;

/** The card's line for its newest activity, or null when nothing has happened in its projects. */
export function activityLine(activity: OrgActivity | null | undefined, now: number = Date.now()): string | null {
	if (!activity) return null;
	const when = timeAgo(activity.at, now);
	const project = activity.project;
	if (activity.kind === 'published') {
		return activity.by ? m.org_activity_published_by({ name: activity.by, project, when }) : m.org_activity_published({ project, when });
	}
	return activity.by ? m.org_activity_worked_by({ name: activity.by, project, when }) : m.org_activity_worked({ project, when });
}

/** "active now", "active 3h ago", or null when they've never been seen. */
export function lastActive(at: string | null | undefined, now: number = Date.now()): string | null {
	if (!at) return null;
	const time = new Date(at).getTime();
	if (Number.isNaN(time)) return null;
	if (now - time < ACTIVE_NOW_MINUTES * MINUTE_MS) return m.org_active_now();
	return m.org_active_ago({ when: timeAgo(at, now) });
}

/** Why the backend wouldn't delete a workspace, from its 409 reason. */
export function deleteRefusal(reason: string): string {
	if (reason.includes('projects')) return m.org_err_delete_projects();
	if (reason.includes('personal')) return m.org_err_delete_personal();
	return m.org_err_delete_only();
}
