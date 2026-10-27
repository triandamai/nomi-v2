import { m } from '$lib/paraglide/messages';

// Admin access is per area at one of three levels; a level includes the ones below it.
// The backend's list is nomi_auth::catalog (`GET /api/admin/permissions/catalog`).

export const LEVELS = ['view', 'write', 'manage'] as const;
export type Level = (typeof LEVELS)[number];

export const LEVEL_LABELS: Record<Level, () => string> = {
	view: () => m.perm_level_read(),
	write: () => m.perm_level_write(),
	manage: () => m.perm_level_manage(),
};

export const LEVEL_HINTS: Record<Level, () => string> = {
	view: () => m.perm_level_read_hint(),
	write: () => m.perm_level_write_hint(),
	manage: () => m.perm_level_manage_hint(),
};

export interface AdminArea {
	resource: string;
	label: () => string;
	description: () => string;
	/** Admin pages the area covers. */
	paths: string[];
}

export const ADMIN_AREAS: AdminArea[] = [
	{ resource: 'dashboard', label: () => m.perm_dashboard(), description: () => m.perm_dashboard_desc(), paths: ['/admin/agents', '/admin/usage'] },
	{ resource: 'user', label: () => m.admin_users(), description: () => m.perm_users_desc(), paths: ['/admin/users'] },
	{ resource: 'plans', label: () => m.admin_plans(), description: () => m.perm_plans_desc(), paths: ['/admin/plans'] },
	{ resource: 'notifications', label: () => m.admin_notifications(), description: () => m.perm_notifications_desc(), paths: ['/admin/notifications'] },
	{ resource: 'models', label: () => m.admin_models(), description: () => m.perm_models_desc(), paths: ['/admin/settings/llm', '/admin/settings/providers', '/admin/settings/embedding'] },
	{ resource: 'tools', label: () => m.admin_tools(), description: () => m.perm_tools_desc(), paths: ['/admin/tools'] },
	{ resource: 'agents', label: () => m.admin_custom_agents(), description: () => m.perm_agents_desc(), paths: ['/admin/dynamic-agents'] },
	{ resource: 'support', label: () => m.admin_support(), description: () => m.perm_support_desc(), paths: ['/admin/support'] },
	{ resource: 'maintenance', label: () => m.admin_maintenance(), description: () => m.perm_maintenance_desc(), paths: ['/admin/maintenance'] },
	{ resource: 'integrations', label: () => m.admin_integrations(), description: () => m.perm_integrations_desc(), paths: ['/admin/integrations'] },
	{ resource: 'configuration', label: () => m.admin_configuration(), description: () => m.perm_configuration_desc(), paths: ['/admin/configuration'] },
];

export const areaFor = (resource: string) => ADMIN_AREAS.find((a) => a.resource === resource);

/** Each admin area's level from someone's permission strings (`nomi:admin:<resource>:[...]`). */
export function adminAccess(permissions: string[]): Record<string, Level> {
	const access: Record<string, Level> = {};
	for (const permission of permissions) {
		const match = permission.match(/^nomi:admin:([a-z0-9_]+):\[(.*)\]$/);
		if (!match) continue;
		const top = highest(match[2].split(','));
		if (top && (!access[match[1]] || rank(top) > rank(access[match[1]]))) access[match[1]] = top;
	}
	return access;
}

export const rank = (level: string) => LEVELS.indexOf(level as Level);

export function highest(actions: string[]): Level | null {
	let best = -1;
	for (const action of actions) best = Math.max(best, rank(action));
	return best >= 0 ? LEVELS[best] : null;
}

/** Whether the access reaches `level` on the area. */
export const can = (access: Record<string, Level>, resource: string, level: Level = 'view') =>
	resource in access && rank(access[resource]) >= rank(level);

/** The area an admin page belongs to; the overview (`/admin`) is the dashboard. */
export function areaForPath(path: string): AdminArea | undefined {
	if (path === '/admin' || path === '/admin/') return ADMIN_AREAS[0];
	return ADMIN_AREAS.find((a) => a.paths.some((p) => path === p || path.startsWith(`${p}/`)));
}

/** Areas whose label, description or key match the search, best first. */
export function searchAreas(query: string, areas: AdminArea[] = ADMIN_AREAS): AdminArea[] {
	const q = query.trim().toLowerCase();
	if (!q) return areas;
	const scored = areas
		.map((area) => {
			const label = area.label().toLowerCase();
			const score = label.startsWith(q) ? 0 : label.includes(q) ? 1 : area.resource.includes(q) ? 2 : area.description().toLowerCase().includes(q) ? 3 : -1;
			return { area, score };
		})
		.filter((s) => s.score >= 0);
	scored.sort((a, b) => a.score - b.score);
	return scored.map((s) => s.area);
}
