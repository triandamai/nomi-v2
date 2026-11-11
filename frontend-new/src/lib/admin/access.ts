import type { PermissionGrant } from './api';

// Who may see and change which part of the admin console. Access is per area, at one of three
// levels, each including the ones below it (nomi_auth::catalog). The person's token carries it as
// `nomi:admin:<area>:[levels]`; a platform admin's carries every area at manage. The backend's
// require_admin stays the real check: this decides only what the console shows.

export const LEVELS = ['view', 'write', 'manage'] as const;
export type Level = (typeof LEVELS)[number];

/** Every admin area, in the backend's console order (ADMIN_RESOURCES). */
export const AREAS = ['dashboard', 'user', 'plans', 'notifications', 'models', 'tools', 'agents', 'support', 'maintenance', 'integrations', 'configuration'] as const;
export type Area = (typeof AREAS)[number];

export type Access = Partial<Record<Area, Level>>;

/** The error message of the console's own "no admin access" page (a 403 from the gate). */
export const NO_ADMIN_ACCESS = 'no_admin_access';

const PERMISSION = /^nomi:admin:([a-z0-9_]+):\[(.*)\]$/;

export const rank = (level: string | null | undefined): number => (level ? LEVELS.indexOf(level as Level) : -1);

const isArea = (value: string): value is Area => (AREAS as readonly string[]).includes(value);

/** The highest level among some actions, or null when none is a level. */
export function highest(actions: string[]): Level | null {
	const best = Math.max(-1, ...actions.map(rank));
	return best >= 0 ? LEVELS[best] : null;
}

/** Each admin area's level, from the permissions in someone's token (`GET /api/whoami`). */
export function adminAccess(permissions: string[]): Access {
	const access: Access = {};
	for (const permission of permissions) {
		const match = permission.match(PERMISSION);
		if (!match || !isArea(match[1])) continue;
		const top = highest(match[2].split(','));
		if (top && rank(top) > rank(access[match[1]])) access[match[1]] = top;
	}
	return access;
}

/** Whether `access` reaches `level` on `area`. */
export const can = (access: Access, area: Area, level: Level = 'view'): boolean => rank(access[area]) >= rank(level);

// The console's pages, the area each belongs to, and how the navigation groups them.

export type PageKey =
	| 'overview'
	| 'live'
	| 'usage'
	| 'users'
	| 'plans'
	| 'notifications'
	| 'models'
	| 'providers'
	| 'embedding'
	| 'tools'
	| 'dynamicAgents'
	| 'support'
	| 'maintenance'
	| 'integrations'
	| 'configuration';

export type AdminPage = { key: PageKey; href: string; area: Area };
export type NavGroupKey = 'dashboard' | 'people' | 'crew' | 'operations';

const GROUPS: { key: NavGroupKey; pages: AdminPage[] }[] = [
	{
		key: 'dashboard',
		pages: [
			{ key: 'overview', href: '/admin', area: 'dashboard' },
			{ key: 'live', href: '/admin/agents', area: 'dashboard' },
			{ key: 'usage', href: '/admin/usage', area: 'dashboard' },
		],
	},
	{
		key: 'people',
		pages: [
			{ key: 'users', href: '/admin/users', area: 'user' },
			{ key: 'plans', href: '/admin/plans', area: 'plans' },
			{ key: 'notifications', href: '/admin/notifications', area: 'notifications' },
		],
	},
	{
		key: 'crew',
		pages: [
			{ key: 'models', href: '/admin/models', area: 'models' },
			{ key: 'providers', href: '/admin/models/providers', area: 'models' },
			{ key: 'embedding', href: '/admin/models/embedding', area: 'models' },
			{ key: 'tools', href: '/admin/tools', area: 'tools' },
			{ key: 'dynamicAgents', href: '/admin/dynamic-agents', area: 'agents' },
		],
	},
	{
		key: 'operations',
		pages: [
			{ key: 'support', href: '/admin/support', area: 'support' },
			{ key: 'maintenance', href: '/admin/maintenance', area: 'maintenance' },
			{ key: 'integrations', href: '/admin/integrations', area: 'integrations' },
			{ key: 'configuration', href: '/admin/configuration', area: 'configuration' },
		],
	},
];

export const PAGES: AdminPage[] = GROUPS.flatMap((group) => group.pages);

/** The page `path` is on (the deepest one that contains it), or null when it isn't an admin page. */
export function pageForPath(path: string): AdminPage | null {
	const clean = path.length > 1 ? path.replace(/\/+$/, '') : path;
	let found: AdminPage | null = null;
	for (const page of PAGES) {
		const inside = page.href === '/admin' ? clean === '/admin' : clean === page.href || clean.startsWith(`${page.href}/`);
		if (inside && (!found || page.href.length > found.href.length)) found = page;
	}
	return found;
}

export const areaForPath = (path: string): Area | null => pageForPath(path)?.area ?? null;

/** The first page someone can see: where the console opens for them. Null when there's none. */
export const homeFor = (access: Access): string | null => PAGES.find((page) => can(access, page.area))?.href ?? null;

/** The navigation's groups, with only the pages someone can see; groups left empty are dropped. */
export function navGroups(access: Access): { key: NavGroupKey; pages: AdminPage[] }[] {
	return GROUPS.map((group) => ({ key: group.key, pages: group.pages.filter((page) => can(access, page.area)) })).filter((group) => group.pages.length > 0);
}

// Giving other people access. The backend refuses a level the granter doesn't hold themselves
// (and checks it against the database, not the token); the console offers no more than that.

/** The levels the granter may give on `area`: up to their own. */
export const grantable = (mine: Access, area: Area): Level[] => LEVELS.filter((level) => can(mine, area, level));

/** Whether the granter may change an area someone holds at `current` (null: none): they need it too. */
export const editable = (mine: Access, area: Area, current: Level | null): boolean => (current ? can(mine, area, current) : can(mine, area, 'view'));

/** Someone's admin role from their grants (`GET /api/admin/users/:id`). Workspace grants are left out. */
export function accessFromGrants(grants: PermissionGrant[]): Access {
	const access: Access = {};
	for (const grant of grants) {
		if (grant.scope_type !== 'admin' || !isArea(grant.resource)) continue;
		const top = highest(grant.actions);
		if (top && rank(top) > rank(access[grant.resource])) access[grant.resource] = top;
	}
	return access;
}

/** The whole role, as `PUT /api/admin/users/:id/admin-access` takes it: areas left out lose access. */
export function accessPayload(access: Partial<Record<Area, Level | null>>): { access: { resource: Area; level: Level }[] } {
	return { access: AREAS.flatMap((area) => (access[area] ? [{ resource: area, level: access[area] }] : [])) };
}
