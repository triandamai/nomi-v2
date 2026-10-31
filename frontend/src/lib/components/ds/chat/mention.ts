import type { Component } from 'svelte';
import type { LucideProps } from '@lucide/svelte';
import type { CrewId } from '../agents/crew';

// Chat responses → Mention Picker. The composer keeps focus while the picker is open, so it moves
// the active option with these keys and points at it with aria-activedescendant.

export type MentionOption =
	| { kind: 'crew'; agent: CrewId; name: string; role: string }
	| { kind: 'file'; name: string; where: string; icon?: Component<LucideProps> };

export function nextOption(active: number, key: string, count: number): number {
	if (count === 0) return active;
	if (key === 'ArrowDown') return (active + 1) % count;
	if (key === 'ArrowUp') return (active - 1 + count) % count;
	if (key === 'Home') return 0;
	if (key === 'End') return count - 1;
	return active;
}
