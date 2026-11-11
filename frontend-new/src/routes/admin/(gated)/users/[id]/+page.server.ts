import { error } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { AREAS, LEVELS, accessPayload, can, type Area, type Level } from '$lib/admin/access';
import type { AdminOrg, AdminPlan, Subscription, UserDetail } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * One person (user: view): their profile, admin role, workspace grants and memberships, and, for
 * someone who may see plans, their plan and allowance. What the viewer can't see isn't asked for;
 * what they can't change is shown without its form (the page decides from their access).
 */
export const load: PageServerLoad = async (event) => {
	const { access } = await event.parent();
	const id = event.params.id;
	const [person, orgs, subscription, plans] = await Promise.all([
		getJson<UserDetail>(event, `/api/admin/users/${id}`),
		can(access, 'user', 'write') ? getJson<AdminOrg[]>(event, '/api/admin/orgs') : null,
		can(access, 'plans', 'view') ? getJson<Subscription>(event, `/api/admin/users/${id}/subscription`) : null,
		can(access, 'plans', 'write') ? getJson<AdminPlan[]>(event, '/api/admin/plans') : null,
	]);
	if (!person.ok && person.problem.status === 404) error(404, { message: m.admin_user_missing() });
	return { title: person.ok ? person.data.email : m.admin_page_users(), person: mustLoad(person), orgs, subscription, plans };
};

const ROLES = ['owner', 'admin', 'member'];

/** The allowance's last day, in UTC, as the backend keeps it: the end of that day. */
function endOfDay(date: string): string | null {
	if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) return null;
	const at = new Date(`${date}T23:59:59.000Z`);
	return Number.isNaN(at.getTime()) || at.toISOString().slice(0, 10) !== date ? null : at.toISOString();
}

/** An admin role from the access editor's JSON: each known area at a known level, or null. */
function readRole(text: string): Partial<Record<Area, Level | null>> | null {
	let parsed: unknown;
	try {
		parsed = JSON.parse(text);
	} catch {
		return null;
	}
	if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return null;
	const role: Partial<Record<Area, Level | null>> = {};
	for (const [area, level] of Object.entries(parsed)) {
		if (!(AREAS as readonly string[]).includes(area)) return null;
		if (level !== null && !(LEVELS as readonly unknown[]).includes(level)) return null;
		role[area as Area] = level as Level | null;
	}
	return role;
}

export const actions: Actions = {
	profile: async (event) => {
		const form = await event.request.formData();
		return act(event, 'profile', `/api/admin/users/${event.params.id}/profile`, 'PUT', { display_name: field(form, 'display_name'), username: field(form, 'username') });
	},

	access: async (event) => {
		const role = readRole(field(await event.request.formData(), 'access'));
		if (!role) return refuse('access', m.admin_user_access_invalid());
		return act(event, 'access', `/api/admin/users/${event.params.id}/admin-access`, 'PUT', accessPayload(role));
	},

	addMembership: async (event) => {
		const form = await event.request.formData();
		const orgId = field(form, 'org_id');
		const role = field(form, 'role');
		if (!orgId) return refuse('addMembership', m.admin_user_workspace_required());
		if (!ROLES.includes(role)) return refuse('addMembership', m.admin_user_role_invalid());
		return act(event, 'addMembership', `/api/admin/users/${event.params.id}/memberships`, 'POST', { org_id: orgId, role });
	},

	removeMembership: async (event) => {
		const orgId = field(await event.request.formData(), 'org_id');
		if (!orgId) return refuse('removeMembership', m.admin_user_workspace_required());
		return act(event, 'removeMembership', `/api/admin/users/${event.params.id}/memberships/${encodeURIComponent(orgId)}`, 'DELETE');
	},

	subscription: async (event) => {
		const form = await event.request.formData();
		const planId = field(form, 'plan_id');
		const quota = field(form, 'quota_override').replace(/[\s,._]/g, '');
		const until = field(form, 'override_until');
		const note = field(form, 'note');
		if (!planId) return refuse('subscription', m.admin_user_plan_required());
		if (quota && !/^\d+$/.test(quota)) return refuse('subscription', m.admin_user_quota_invalid());
		const overrideUntil = until ? endOfDay(until) : null;
		if (until && !overrideUntil) return refuse('subscription', m.admin_user_until_invalid());
		return act(event, 'subscription', `/api/admin/users/${event.params.id}/subscription`, 'PUT', {
			plan_id: planId,
			quota_override: quota ? Number(quota) : null,
			override_until: overrideUntil,
			note: note || null,
		});
	},

	revoke: async (event) => {
		const permissionId = field(await event.request.formData(), 'permission_id');
		if (!permissionId) return refuse('revoke', m.admin_user_grant_required());
		return act(event, 'revoke', `/api/admin/users/${event.params.id}/permissions/${encodeURIComponent(permissionId)}`, 'DELETE');
	},
};
