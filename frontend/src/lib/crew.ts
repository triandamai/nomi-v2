// Who's in this chat's crew and what each member is doing right now — derived from the data the
// chat page already has (the app-wide roster from /api/agents, delegations, message authors,
// Nomi's live phase). No extra API calls.

import { agentTypeFallbackLabel } from './agentLabels';
import { agentLook } from './components/m3/shapes';

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

const CORE: { key: string; name: string; role: string }[] = [
	{ key: 'nomi', name: 'Nomi', role: 'Talks with you and routes the work' },
	{ key: 'money', name: 'Money', role: 'Transactions, budgets, subscriptions' },
	{ key: 'coding', name: 'Coding', role: 'Builds and edits project files' },
	{ key: 'planning', name: 'Planning', role: 'Plans, to-dos and reminders' },
	{ key: 'personality', name: 'Personality', role: 'Keeps Nomi sounding how you like' },
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
	const core = CORE.find((m) => m.key !== 'nomi' && agentLook(lower).shape === agentLook(m.key).shape);
	return core ? core.key : lower;
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
	const seed = roster.length > 0 ? roster.map((m) => ({ key: rosterKey(m.agent_type), name: m.name, role: m.role })) : CORE;
	for (const member of seed) {
		members.set(member.key, { ...member, working: false, status: 'Standing by', involved: false });
	}

	const ensure = (agent: string): CrewMember => {
		const key = memberKey(agent, roster);
		let member = members.get(key);
		if (!member) {
			// A dynamic agent this chat has met — joins the crew after the core members.
			member = { key, name: agentTypeFallbackLabel(agent).replace(/_/g, ' '), role: 'Custom agent', working: false, status: 'Standing by', involved: false };
			members.set(key, member);
		}
		return member;
	};

	for (const author of messageAuthors) {
		const member = author ? ensure(author) : members.get('nomi')!;
		if (!member.involved) {
			member.involved = true;
			member.status = 'Replied in this chat';
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
					? `Couldn't finish: ${clip(item.task, 48)}`
					: item.status === 'cancelled'
						? `Stopped: ${clip(item.task, 51)}`
						: `Done: ${clip(item.task, 54)}`;
		}
	}

	const nomi = members.get('nomi')!;
	nomi.involved = true;
	if (nomiWorking) {
		nomi.working = true;
		nomi.status = nomiStatus ?? 'Working…';
	} else if (nomi.status === 'Standing by') {
		nomi.status = 'Ready when you are';
	}

	return [...members.values()];
}
