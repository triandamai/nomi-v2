import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { OrgRole } from '$lib/adapters/api';
import { WORKSPACE_NAME_MAX } from '$lib/server/account';
import * as workspaces from '$lib/server/workspaces';
import { parseEmails } from '$lib/settings/workspace';
import type { Actions, PageServerLoad } from './$types';

/** Settings › Workspace (App / Settings / Workspace wZm9B): one workspace, its people and invites. */
export const load: PageServerLoad = async (event) => {
	event.depends('app:workspace');
	const data = await workspaces.loadWorkspace(event, event.params.id);
	if (!data.detail) error(data.status === 503 ? 503 : 404, { message: data.status === 503 ? m.workspaces_load_failed() : m.workspace_not_found() });
	const people = data.detail.members.length;
	return { title: data.detail.name, meta: people === 1 ? m.workspace_only_you() : m.workspace_members({ count: people }), ...data };
};

const field = (form: FormData, name: string) => String(form.get(name) ?? '').trim();
const ROLES: OrgRole[] = ['owner', 'admin', 'member'];
const refused = (sent: { status: number; message: string }) => fail(sent.status, { message: sent.message || m.action_failed() });

export const actions: Actions = {
	rename: async (event) => {
		const name = field(await event.request.formData(), 'name');
		if (!name) return fail(400, { field: 'name', message: m.workspace_name_required() });
		if ([...name].length > WORKSPACE_NAME_MAX) return fail(400, { field: 'name', message: m.workspace_name_long() });
		const sent = await workspaces.rename(event, event.params.id, name);
		return sent.ok ? { renamed: name } : fail(sent.status, { field: 'name', message: sent.message || m.action_failed() });
	},
	/** Each address in turn; the ones the backend turned away come back with its reason. */
	invite: async (event) => {
		const form = await event.request.formData();
		const { valid, invalid } = parseEmails(field(form, 'emails'));
		if (invalid.length) return fail(400, { field: 'emails', message: m.workspace_invite_check({ emails: invalid.join(', ') }) });
		if (!valid.length) return fail(400, { field: 'emails', message: m.workspace_invite_needed() });
		const role = field(form, 'role') === 'admin' ? 'admin' : 'member';
		const failed: { email: string; message: string }[] = [];
		for (const email of valid) {
			const sent = await workspaces.invite(event, event.params.id, email, role);
			if (!sent.ok) failed.push({ email, message: sent.message || m.action_failed() });
		}
		return { sent: valid.length - failed.length, failed };
	},
	revoke: async (event) => {
		const code = field(await event.request.formData(), 'code');
		if (!code) return fail(400, { message: m.action_failed() });
		const sent = await workspaces.revoke(event, event.params.id, code);
		return sent.ok ? { revoked: code } : refused(sent);
	},
	role: async (event) => {
		const form = await event.request.formData();
		const [user, role] = [field(form, 'user'), field(form, 'role') as OrgRole];
		if (!user || !ROLES.includes(role)) return fail(400, { message: m.action_failed() });
		const sent = await workspaces.setRole(event, event.params.id, user, role);
		return sent.ok ? { changed: user } : refused(sent);
	},
	remove: async (event) => {
		const user = field(await event.request.formData(), 'user');
		if (!user) return fail(400, { message: m.action_failed() });
		const sent = await workspaces.removeMember(event, event.params.id, user);
		return sent.ok ? { removed: user } : refused(sent);
	},
	/** The person leaves; the workspaces list says so. */
	leave: async (event) => {
		const data = await workspaces.loadWorkspace(event, event.params.id);
		if (!data.detail) return fail(404, { message: m.workspace_not_found() });
		const sent = await workspaces.removeMember(event, event.params.id, data.detail.me);
		if (!sent.ok) return refused(sent);
		redirect(303, `/settings/workspaces?left=${encodeURIComponent(data.detail.name)}`);
	},
};
