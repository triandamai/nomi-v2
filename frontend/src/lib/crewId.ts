// Which design-system crew member an agent is: the twins draw people by CrewId ('koda', 'dana'…),
// the API names agents by agent_type ('coding', 'money'…) or, in older messages, by crew name.
import { CREW, type CrewId } from '$lib/components/ds/agents/crew';
import { CREW_NAMES } from '$lib/crewNames';

const IDS = new Set<string>(CREW.map((member) => member.id));

/** The crew member for an agent_type or a crew name; anyone else (chitchat included) is Nomi. */
export function crewId(agent: string | null | undefined): CrewId {
	const key = (agent ?? '').toLowerCase();
	const name = (CREW_NAMES[key] ?? key).toLowerCase();
	return (IDS.has(name) ? name : 'nomi') as CrewId;
}
