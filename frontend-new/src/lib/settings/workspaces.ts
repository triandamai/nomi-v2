import { m } from '$lib/paraglide/messages';
import { workspaceRows } from '$lib/adapters/shell';
import type { OrgList, OrgRole } from '$lib/adapters/api';

// Settings › Workspaces (App / Settings / Workspaces l222Mz): each workspace a card with its tile
// (the switcher's tones, so a workspace looks the same in both), the person's role and who's in
// it, its projects and members, and whether projects come from it (the backend's one choice of
// workspace: there's no active one to switch to).

export type WorkspaceCardData = { id: string; name: string; initial: string; tone: 'plain' | 'teal' | 'amber'; meta: string; projects: number; members: number; current: boolean; href: string };

export const ROLE_NAME: Record<OrgRole, () => string> = { owner: m.workspace_role_owner, admin: m.workspace_role_admin, member: m.workspace_role_member };

export function workspaceCards(list: OrgList): WorkspaceCardData[] {
	const rows = workspaceRows(list);
	return list.orgs.map((org, index) => ({
		id: org.id,
		name: org.name,
		initial: [...org.name.trim()][0]?.toUpperCase() ?? '?',
		tone: rows[index].tone,
		meta: `${ROLE_NAME[org.role]?.() ?? org.role} · ${org.members === 1 ? m.workspace_only_you() : m.workspace_members({ count: org.members })}`,
		projects: org.projects,
		members: org.members,
		current: org.id === list.project_org_id,
		href: `/settings/workspaces/${org.id}`,
	}));
}
