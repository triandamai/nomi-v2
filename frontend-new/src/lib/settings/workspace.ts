import { m } from '$lib/paraglide/messages';
import { getLocale } from '$lib/paraglide/runtime';
import { ago } from '$lib/adapters/time';
import { maskAddress } from '$lib/settings/connections';
import { dayMonth } from '$lib/settings/profile';
import type { OrgDetail, OrgInvite, OrgRole } from '$lib/adapters/api';

// Settings › Workspace (App / Settings / Workspace wZm9B): the backend's rules, so the page never
// offers what it would refuse. Owners and admins invite, take invites back, move people between
// member and admin and remove them; only owners make, unmake or remove owners; nobody changes or
// removes themselves here (they leave); the last owner can't go.

const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@.]+$/;

/** The invite box's addresses: split on commas, semicolons, spaces and new lines, each once. */
export function parseEmails(text: string): { valid: string[]; invalid: string[] } {
	const valid: string[] = [];
	const invalid: string[] = [];
	for (const raw of text.split(/[\s,;]+/)) {
		const email = raw.trim().toLowerCase();
		if (!email) continue;
		const into = EMAIL.test(email) ? valid : invalid;
		if (!into.includes(email)) into.push(email);
	}
	return { valid, invalid };
}

export const isManager = (role: OrgRole) => role === 'owner' || role === 'admin';

export type MemberRow = { id: string; name: string; avatar: string | null; you: boolean; role: OrgRole; meta: string; roleOptions: OrgRole[]; canRemove: boolean };

export function memberRows(detail: OrgDetail, now: Date): MemberRow[] {
	const language = getLocale();
	const mine = detail.my_role;
	return detail.members.map((member) => {
		const you = member.user_id === detail.me;
		const reachable = isManager(mine) && !you && (member.role !== 'owner' || mine === 'owner');
		const options: OrgRole[] = mine === 'owner' ? ['owner', 'admin', 'member'] : ['admin', 'member'];
		const joined = m.workspace_joined({ date: dayMonth(new Date(member.joined_at), language) });
		return {
			id: member.user_id,
			name: member.name,
			avatar: member.avatar_url,
			you,
			role: member.role,
			meta: member.email ? `${maskAddress(member.email)} · ${joined}` : joined,
			roleOptions: reachable ? options : [],
			canRemove: reachable,
		};
	});
}

export type InviteRow = { code: string; email: string; role: OrgRole; meta: string };

export function inviteRows(invites: OrgInvite[], now: Date): InviteRow[] {
	return invites.map((invite) => {
		const days = Math.max(1, Math.ceil((Date.parse(invite.expires_at) - now.getTime()) / 86_400_000));
		return {
			code: invite.code,
			email: invite.email ?? '—',
			role: invite.role,
			meta: `${now.getTime() - Date.parse(invite.created_at) < 60_000 ? m.workspace_invited_now() : m.workspace_invited({ when: ago(invite.created_at, now) })} · ${days === 1 ? m.workspace_expires_one() : m.workspace_expires({ count: days })}`,
		};
	});
}

/** Anyone can leave, except the workspace's last owner. */
export function canLeave(detail: OrgDetail): boolean {
	return detail.my_role !== 'owner' || detail.members.filter((member) => member.role === 'owner').length > 1;
}
