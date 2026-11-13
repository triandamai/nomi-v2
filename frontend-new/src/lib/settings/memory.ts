import type { MemoryItem } from '$lib/adapters/api';

// Settings › Memory & context (App / Settings / Memory hANTE): what the crew remembers, by kind
// (the backend's preference, person, routine, goal and fact stand in for the pen's topics), each
// memory's strength in five steps of its weight (0.1–5, every memory starting at 1: Steady), and
// the Core and Fading views.

export type StrengthName = 'faint' | 'weak' | 'steady' | 'strong' | 'core';
export type Strength = { level: 1 | 2 | 3 | 4 | 5; name: StrengthName };
export type MemoryFilter = 'all' | 'core' | 'fading';
export type MemoryTopic = { kind: string; memories: MemoryItem[]; summary: string; strength: Strength; updated: string; fading: boolean };

export function strength(weight: number): Strength {
	if (weight >= 3) return { level: 5, name: 'core' };
	if (weight >= 1.5) return { level: 4, name: 'strong' };
	if (weight >= 0.9) return { level: 3, name: 'steady' };
	if (weight >= 0.5) return { level: 2, name: 'weak' };
	return { level: 1, name: 'faint' };
}

const isCore = (memory: MemoryItem) => memory.weight >= 3;
/** Fading: faint or weak, or one the crew asks to be checked. */
const isFading = (memory: MemoryItem) => memory.weight < 0.9 || memory.needs_check;

export function memoryTopics(memories: MemoryItem[], filter: MemoryFilter, query: string): MemoryTopic[] {
	const wanted = query.trim().toLowerCase();
	const shown = memories.filter(
		(memory) => (filter === 'all' || (filter === 'core' ? isCore(memory) : isFading(memory))) && (!wanted || memory.content.toLowerCase().includes(wanted)),
	);
	const kinds = new Map<string, MemoryItem[]>();
	for (const memory of shown) kinds.set(memory.kind, [...(kinds.get(memory.kind) ?? []), memory]);
	return [...kinds]
		.map(([kind, items]) => {
			const sorted = [...items].sort((a, b) => b.weight - a.weight);
			const weight = sorted.reduce((sum, memory) => sum + memory.weight, 0) / sorted.length;
			return {
				kind,
				memories: sorted,
				summary: sorted.slice(0, 3).map((memory) => memory.content).join('; '),
				strength: strength(weight),
				updated: sorted.map((memory) => memory.updated_at).sort().at(-1)!,
				fading: sorted.every(isFading),
			};
		})
		.sort((a, b) => b.strength.level - a.strength.level || b.memories.length - a.memories.length);
}

export function memoryCounts(memories: MemoryItem[]): { all: number; core: number; fading: number; topics: number } {
	return { all: memories.length, core: memories.filter(isCore).length, fading: memories.filter(isFading).length, topics: new Set(memories.map((memory) => memory.kind)).size };
}
