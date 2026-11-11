import { m } from '$lib/paraglide/messages';
import type { CrewId } from '$lib/ds/agents/crew';
import type { CrewMember, OrgList, OrgRole, OrgSummary, Profile, SessionSummary, UsageBrief } from './api';
import { ago } from './time';
import { pinnedFirst } from '$lib/chat/sessionOrder';

// The app shell's data, turned from the API's shapes into the design system's props: the sidebar's
// recents and counts, the usage card, the crew panel and the workspace switcher. Pure: the layout
// load runs them on the server, so the page and its hydration agree.

/** What to call the person: the name they gave, else their username, else their address's name. */
export function personName(profile: Profile): string {
	return profile.display_name?.trim() || profile.username?.trim() || profile.email.split('@')[0];
}

export function firstName(name: string): string {
	return name.trim().split(/\s+/)[0] ?? name;
}

/** Token counts as the pen writes them: 950, 21.8K, 1M, 2.5M (one decimal, never rounded up past it). */
export function formatTokens(tokens: number): string {
	// Whole tenths first, so 21 800 is 21.8 and not 21.799999.
	const tenths = (unit: number) => Math.floor(tokens / (unit / 10)) / 10;
	if (tokens >= 999_950) return `${Math.max(tenths(1_000_000), 1)}M`;
	if (tokens >= 1000) return `${tenths(1000)}K`;
	return String(tokens);
}

/** The sidebar's usage card: "Free plan", the share used, "21.8K / 1M tokens this month". */
export function usageCard(brief: UsageBrief): { plan: string; used: number; caption: string } {
	const quota = brief.plan.monthly_tokens;
	return {
		plan: m.usage_plan({ plan: brief.plan.name }),
		used: quota > 0 ? Math.min(brief.tokens_used / quota, 1) : 0,
		caption: m.usage_caption({ used: formatTokens(brief.tokens_used), total: formatTokens(quota) }),
	};
}

/** The account menu's line under the name: "Free · 21.8K / 1M tokens". */
export function accountLine(brief: UsageBrief): string {
	return m.usage_account({ plan: brief.plan.name, used: formatTokens(brief.tokens_used), total: formatTokens(brief.plan.monthly_tokens) });
}

export type Recent = { href: string; kind: 'chat' | 'project'; title: string; when: string; active?: boolean };

/** Where a chat lives: a project's own chat opens the project. */
export function chatHref(session: Pick<SessionSummary, 'id' | 'project_id'>): string {
	return session.project_id ? `/projects/${session.project_id}` : `/chat/${session.id}`;
}

/** The sidebar's recents: the latest chats and projects (the API lists them latest first), the
 * pinned ones on top. */
export function recents(sessions: SessionSummary[], now: Date, current?: string, limit = 4, timeZone?: string): Recent[] {
	return pinnedFirst(sessions).slice(0, limit).map((session) => {
		const href = chatHref(session);
		return {
			href,
			kind: session.project_id ? 'project' : 'chat',
			title: session.title?.trim() || m.recents_untitled(),
			when: ago(session.last_message?.created_at ?? session.updated_at, now, timeZone),
			...(href === current && { active: true }),
		};
	});
}

/** The Chats count: the person's chats, not the projects' own ones. */
export function chatsCount(sessions: SessionSummary[]): number {
	return sessions.filter((session) => !session.project_id).length;
}

/** Which mascot each built-in agent wears (backend: nomi_agent_core::crew::crew_name). */
const MASCOT: Record<string, CrewId> = {
	chitchat: 'nomi',
	money: 'dana',
	reminders: 'kala',
	files: 'maya',
	planning: 'rena',
	coding: 'koda',
	workspace: 'tara',
	health: 'raga',
	personality: 'personality',
	supervisor: 'supervisor',
};

/** The mascot a built-in agent wears; none for a custom one. */
export const crewOf = (agentType: string): CrewId | undefined => MASCOT[agentType];

export type CrewRowProps = { agent?: CrewId; name: string; role: string; state?: string };

const isBusy = (member: CrewMember) => member.state === 'working' || member.state === 'waiting';

/** The crew panel's rows: each member's mascot (none for one the design has no mascot for), what
 * they do, and what they're doing when it isn't nothing. */
export function crewRows(members: CrewMember[]): CrewRowProps[] {
	return members.map((member) => {
		const agent = member.is_dynamic ? undefined : MASCOT[member.agent_type];
		const state = member.state === 'working' ? m.crew_state_working() : member.state === 'waiting' ? m.crew_state_waiting() : undefined;
		return { ...(agent && { agent }), name: member.name, role: member.role, ...(state && { state }) };
	});
}

/** The crew panel's head: "10 idle", or "2 working · 8 idle" while some are at it. */
export function crewSummary(members: CrewMember[]): { kind: 'idle' | 'live'; label: string } {
	const busy = members.filter(isBusy).length;
	const idle = members.length - busy;
	return busy > 0 ? { kind: 'live', label: m.crew_summary_working({ working: busy, idle }) } : { kind: 'idle', label: m.crew_summary_idle({ idle }) };
}

export type WorkspaceRow = { id: string; name: string; tone: 'plain' | 'teal' | 'amber'; meta: string; chosen: boolean };

const ROLE: Record<OrgRole, () => string> = { owner: m.workspace_role_owner, admin: m.workspace_role_admin, member: m.workspace_role_member };

/**
 * The workspace switcher's rows: a tile (the first plain, then teal and amber in turn, as the pen
 * draws them; the API gives workspaces no colour), the role, who's in it and how many projects,
 * and a check on the one projects are listed from.
 */
export function workspaceRows(list: OrgList): WorkspaceRow[] {
	return list.orgs.map((org, index) => ({
		id: org.id,
		name: org.name,
		tone: index === 0 ? 'plain' : index % 2 === 1 ? 'teal' : 'amber',
		meta: [
			ROLE[org.role]?.() ?? org.role,
			org.members === 1 ? m.workspace_only_you() : m.workspace_members({ count: org.members }),
			org.projects === 1 ? m.workspace_projects_one() : m.workspace_projects({ count: org.projects }),
		].join(' · '),
		chosen: org.id === list.project_org_id,
	}));
}

/** The workspace projects are listed from (the Build group's chip), or null until one is picked. */
export function projectsWorkspace(list: OrgList): OrgSummary | null {
	return list.orgs.find((org) => org.id === list.project_org_id) ?? null;
}
