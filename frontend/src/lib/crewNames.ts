// The names people see for the built-in crew, by agent_type (mirrors
// backend/crates/nomi-agent-core/src/crew.rs). Code and data keep using the agent_type.
export const CREW_NAMES: Record<string, string> = {
	chitchat: 'Nomi',
	money: 'Dana',
	reminders: 'Kala',
	files: 'Maya',
	planning: 'Rena',
	coding: 'Koda',
	workspace: 'Tara',
	health: 'Raga',
	inspector: 'Iris',
};

/** The crew name for an agent_type (its type when it has none). */
export function crewName(agentType: string): string {
	return CREW_NAMES[agentType] ?? agentType;
}

/** agent_type by crew name, lowercased ("dana" → "money"). */
export const CREW_TYPE_BY_NAME: Record<string, string> = Object.fromEntries(
	Object.entries(CREW_NAMES).map(([type, name]) => [name.toLowerCase(), type]),
);
