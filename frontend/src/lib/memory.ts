// Pure helpers for the memory page (kept out of the component so they're unit-testable).

/** memory_items.weight is clamped to [0.1, 5.0] and starts at 1.0 (see nomi-agent-core's memory.rs). */
const MIN_WEIGHT = 0.1;
const MAX_WEIGHT = 5;

/** How strong a memory reads for its 1–5 pips. */
export function strengthLabel(pips: number): string {
	return [m.mem_strength_faint, m.mem_strength_light, m.mem_strength_steady, m.mem_strength_strong, m.mem_strength_core][Math.min(5, Math.max(1, pips)) - 1]();
}

/**
 * Weight as 1–5 pips on a log scale, so the starting weight (1.0) reads as a middling 3 and each
 * thumbs-up (×1.2) or thumbs-down (×0.8) nudges it visibly but not wildly.
 */
export function strengthPips(weight: number): number {
	const clamped = Math.min(MAX_WEIGHT, Math.max(MIN_WEIGHT, weight));
	const ratio = Math.log(clamped / MIN_WEIGHT) / Math.log(MAX_WEIGHT / MIN_WEIGHT);
	return Math.min(5, Math.max(1, Math.round(ratio * 5)));
}

import { m } from '$lib/paraglide/messages';

export { timeAgo } from '$lib/i18n';
import type { MemoryKind } from '$lib/types';

export const MEMORY_KINDS: MemoryKind[] = ['preference', 'person', 'routine', 'goal', 'fact'];

/** What kind of memory it is, in the reader's language. */
export function kindLabel(kind: MemoryKind): string {
	return { preference: m.mem_kind_preference, person: m.mem_kind_person, routine: m.mem_kind_routine, goal: m.mem_kind_goal, fact: m.mem_kind_fact }[kind]();
}

/**
 * Fits 2D points into a `size`×`size` box with `padding` on every side, keeping their aspect
 * ratio. A single point (or all-identical points) lands in the middle.
 */
export function fitPoints(points: { x: number; y: number }[], size: number, padding: number): { x: number; y: number }[] {
	if (points.length === 0) return [];
	const xs = points.map((p) => p.x);
	const ys = points.map((p) => p.y);
	const minX = Math.min(...xs);
	const minY = Math.min(...ys);
	const span = Math.max(Math.max(...xs) - minX, Math.max(...ys) - minY);
	const inner = size - padding * 2;
	if (span < 1e-9) return points.map(() => ({ x: size / 2, y: size / 2 }));
	const offsetX = (inner - ((Math.max(...xs) - minX) / span) * inner) / 2;
	const offsetY = (inner - ((Math.max(...ys) - minY) / span) * inner) / 2;
	return points.map((p) => ({
		x: padding + offsetX + ((p.x - minX) / span) * inner,
		y: padding + offsetY + ((p.y - minY) / span) * inner,
	}));
}
