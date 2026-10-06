// Who's in this chat's crew and what each member is doing right now — derived from the data the
// chat page already has (the app-wide roster from /api/agents, delegations, message authors,
// Nomi's live phase). No extra API calls.

import { agentTypeFallbackLabel } from './agentLabels';
import { agentLook } from './components/m3/shapes';
import { m } from '$lib/paraglide/messages';

export interface CrewActivity {
	id: string;
	target_agent_type: string;
	status: string;
	task: string;
	result: string | null;
	error: string | null;
}

export interface CrewMember {
	/** Stable key, also passed to AgentShape as `agent`. */
	key: string;
	name: string;
	role: string;
	working: boolean;
	/** Short present-tense line: what it's doing, or what it last did. */
	status: string;
	/** True when this member has spoken or been handed work in this chat. */
	involved: boolean;
}

const core = (): { key: string; name: string; role: string }[] => [
	{ key: 'nomi', name: 'Nomi', role: m.crew_role_nomi() },
	{ key: 'money', name: 'Money', role: m.crew_role_money() },
	{ key: 'coding', name: 'Coding', role: m.crew_role_coding() },
	{ key: 'planning', name: 'Planning', role: m.crew_role_planning() },
	{ key: 'personality', name: 'Personality', role: m.crew_role_personality() },
];

/** One agent from /api/agents. */
export interface CrewRosterMember {
	agent_type: string;
	name: string;
	role: string;
	is_dynamic: boolean;
	shape: string | null;
	tone: string | null;
	motion: string | null;
	/** What it's doing for this user right now, across all chats. */
	state: 'working' | 'waiting' | 'done' | 'idle';
	status: string;
}

/** The crew key for an agent type: Nomi's is `nomi` (AgentShape's brand mark), others their type. */
export function rosterKey(agentType: string): string {
	return agentType === 'chitchat' ? 'nomi' : agentType;
}

const ACTIVE_STATUSES = new Set(['pending', 'processing', 'claimed']);

function memberKey(agent: string, roster: CrewRosterMember[]): string {
	const lower = agent.toLowerCase();
	const known = roster.find((m) => m.agent_type.toLowerCase() === lower || m.name.toLowerCase() === lower);
	if (known) return rosterKey(known.agent_type);
	if (lower === 'chitchat' || lower === 'nomi' || lower === 'supervisor') return 'nomi';
	const match = core().find((c) => c.key !== 'nomi' && agentLook(lower).shape === agentLook(c.key).shape);
	return match ? match.key : lower;
}

function clip(text: string, max = 64): string {
	const flat = text.replace(/\s+/g, ' ').trim();
	return flat.length > max ? `${flat.slice(0, max - 1)}…` : flat;
}

export function buildCrew({
	activity,
	messageAuthors,
	nomiWorking,
	nomiStatus,
	roster = [],
}: {
	/** Newest first, as /agent-activity returns it. */
	activity: CrewActivity[];
	/** agent_display_name of each assistant message (null = Nomi). */
	messageAuthors: (string | null)[];
	nomiWorking: boolean;
	/** Nomi's live phase line, e.g. "thinking…". */
	nomiStatus: string | null;
	/** Every agent the app has (from /api/agents). Without it, the five core members stand in. */
	roster?: CrewRosterMember[];
}): CrewMember[] {
	const members = new Map<string, CrewMember>();
	const seed = roster.length > 0 ? roster.map((m) => ({ key: rosterKey(m.agent_type), name: m.name, role: m.role })) : core();
	for (const member of seed) {
		members.set(member.key, { ...member, working: false, status: m.crew_standing_by(), involved: false });
	}

	const ensure = (agent: string): CrewMember => {
		const key = memberKey(agent, roster);
		let member = members.get(key);
		if (!member) {
			// A dynamic agent this chat has met — joins the crew after the core members.
			member = { key, name: agentTypeFallbackLabel(agent).replace(/_/g, ' '), role: m.crew_role_custom(), working: false, status: m.crew_standing_by(), involved: false };
			members.set(key, member);
		}
		return member;
	};

	for (const author of messageAuthors) {
		const member = author ? ensure(author) : members.get('nomi')!;
		if (!member.involved) {
			member.involved = true;
			member.status = m.crew_replied();
		}
	}

	// Oldest → newest so the latest delegation wins each member's status line.
	for (const item of [...activity].reverse()) {
		const member = ensure(item.target_agent_type);
		member.involved = true;
		if (ACTIVE_STATUSES.has(item.status)) {
			member.working = true;
			member.status = clip(item.task);
		} else if (!member.working) {
			member.status =
				item.status === 'failed'
					? m.crew_couldnt_finish({ task: clip(item.task, 48) })
					: item.status === 'cancelled'
						? m.crew_stopped({ task: clip(item.task, 51) })
						: m.crew_done({ task: clip(item.task, 54) });
		}
	}

	const nomi = members.get('nomi')!;
	nomi.involved = true;
	if (nomiWorking) {
		nomi.working = true;
		nomi.status = nomiStatus ?? m.status_working();
	} else if (nomi.status === m.crew_standing_by()) {
		nomi.status = m.crew_ready();
	}

	return [...members.values()];
}

const STATE_ORDER: Record<CrewRosterMember['state'], number> = { working: 0, waiting: 1, done: 2, idle: 3 };

/** The crew, busiest first: working, then waiting on you, then done, then ready. Ties keep the
 * roster's order (Nomi first). */
export function crewByActivity(crew: CrewRosterMember[]): CrewRosterMember[] {
	return crew
		.map((member, index) => ({ member, index }))
		.sort((a, b) => STATE_ORDER[a.member.state] - STATE_ORDER[b.member.state] || a.index - b.index)
		.map(({ member }) => member);
}

/** Home's crew card: the `max` busiest members, and how many more the "See all" link leads to. */
export function crewPreview(crew: CrewRosterMember[], max = 4): { shown: CrewRosterMember[]; more: number } {
	const sorted = crewByActivity(crew);
	return { shown: sorted.slice(0, max), more: Math.max(0, sorted.length - max) };
}
