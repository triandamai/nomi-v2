import { m } from '$lib/paraglide/messages';
import type { CrewMember } from '$lib/adapters/api';

// Settings › Crew (App / Settings / Crew vVUeh): the crew as the backend has it, in the pen's
// groups. CORE is Nomi and the Supervisor, always on; SPECIALISTS the other built-in members;
// CUSTOM the ones an admin made. A member is chosen by its agent type (`?member=`).

export type CrewGroupKey = 'core' | 'specialists' | 'custom';
export type CrewGroup = { key: CrewGroupKey; members: CrewMember[] };

const CORE = ['chitchat', 'supervisor'];

export const groupOf = (member: CrewMember): CrewGroupKey => (member.is_dynamic ? 'custom' : CORE.includes(member.agent_type) ? 'core' : 'specialists');

export function crewGroups(members: CrewMember[], query: string): CrewGroup[] {
	const wanted = query.trim().toLowerCase();
	const found = members.filter((member) => !wanted || member.name.toLowerCase().includes(wanted) || member.role.toLowerCase().includes(wanted));
	const ordered = (key: CrewGroupKey) => {
		const inGroup = found.filter((member) => groupOf(member) === key);
		return key === 'core' ? inGroup.sort((a, b) => CORE.indexOf(a.agent_type) - CORE.indexOf(b.agent_type)) : inGroup;
	};
	return (['core', 'specialists', 'custom'] as const).map((key) => ({ key, members: ordered(key) })).filter((group) => group.members.length > 0);
}

/** The member asked for, or Nomi (the first) when there's no such member. */
export function chosenMember(members: CrewMember[], type: string | null): CrewMember | null {
	return members.find((member) => member.agent_type === type) ?? members.find((member) => member.agent_type === 'chitchat') ?? members[0] ?? null;
}

/** The top bar's "11 members · 2 working". */
export function crewMeta(members: CrewMember[]): string {
	const busy = members.filter((member) => member.state === 'working' || member.state === 'waiting').length;
	const count = members.length === 1 ? m.crew_meta_one() : m.crew_meta({ count: members.length });
	return busy ? `${count} · ${m.crew_meta_working({ count: busy })}` : count;
}

/** Whose model is the person's to pick: Nomi answers on the chat model, Koda builds on the coding one. */
export function modelRole(agentType: string): 'chat' | 'coding' | null {
	return agentType === 'chitchat' ? 'chat' : agentType === 'coding' ? 'coding' : null;
}
