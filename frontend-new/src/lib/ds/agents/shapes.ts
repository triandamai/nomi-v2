// The crew's agent shapes, as frontend draws them (frontend/src/lib/components/m3/shapes.ts):
// each member is a "polar" outline r(θ) = R·(1 + amplitude·cos(lobes·(θ − phase))) in a 48×48
// view, filled with their gradient. `motion` is how frontend moves each one while it works; Mascot
// doesn't take `working` yet, since frontend's twin would have to take it too. The pen's flat
// mascots are kept in $lib/rollback/PenMascot.svelte (with ./mascots.ts) to roll back to.

import type { CrewId } from './crew';

export type AgentShapeName = 'cookie9' | 'cookie6' | 'sunny8' | 'sunny12' | 'clover3' | 'clover4' | 'flower5' | 'puffy7' | 'burst16' | 'soft-square';
export type AgentTone = 'glow' | 'ember' | 'tide' | 'sky' | 'bloom' | 'dusk' | 'citrus' | 'slate';
export type AgentMotion = 'spin' | 'wobble' | 'bounce' | 'pulse' | 'orbit';
export type AgentLook = { shape: AgentShapeName; tone: AgentTone; motion: AgentMotion };

/** The side of the square view the outlines are drawn in. */
export const SHAPE_VIEW = 48;
const CENTER = SHAPE_VIEW / 2;
const RADIUS = 20;
// 120 keeps the outline smooth at the largest size (96).
const POINTS = 120;
const QUARTER_TURN = Math.PI / 2;
const EIGHTH_TURN = Math.PI / 4;
const DECIMALS = 2;

const SPECS: Record<AgentShapeName, { lobes: number; amplitude: number; phase?: number }> = {
	cookie9: { lobes: 9, amplitude: 0.09 },
	cookie6: { lobes: 6, amplitude: 0.1 },
	sunny8: { lobes: 8, amplitude: 0.06 },
	sunny12: { lobes: 12, amplitude: 0.045 },
	clover3: { lobes: 3, amplitude: 0.19, phase: -QUARTER_TURN },
	clover4: { lobes: 4, amplitude: 0.2 },
	flower5: { lobes: 5, amplitude: 0.16 },
	puffy7: { lobes: 7, amplitude: 0.13 },
	burst16: { lobes: 16, amplitude: 0.055 },
	'soft-square': { lobes: 4, amplitude: 0.075, phase: EIGHTH_TURN },
};

/** The outline of a shape as an SVG path in the 48×48 view. */
export function shapePath(name: AgentShapeName): string {
	const { lobes, amplitude, phase = 0 } = SPECS[name];
	const points: string[] = [];
	for (let i = 0; i < POINTS; i++) {
		const theta = (2 * Math.PI * i) / POINTS;
		const r = RADIUS * (1 + amplitude * Math.cos(lobes * (theta - phase)));
		points.push(`${(CENTER + r * Math.cos(theta)).toFixed(DECIMALS)} ${(CENTER + r * Math.sin(theta)).toFixed(DECIMALS)}`);
	}
	return `M${points.join(' L')}Z`;
}

/** Each tone's gradient stops, frontend's --nomi-gradient-* values (an SVG fill can't read a CSS
 * gradient). They are the shape's own paint, the same in both themes. */
export const TONE_STOPS: Record<AgentTone, string[]> = {
	glow: ['#e4f76a', '#5be08f', '#1db9a0'],
	ember: ['#ffd27a', '#ff7a4a'],
	tide: ['#c4f3ea', '#3fb8c8'],
	sky: ['#d6ecff', '#7ab0ff'],
	bloom: ['#ffd9e2', '#ff8fa8'],
	dusk: ['#e6dcff', '#9b7bff'],
	citrus: ['#fff4a3', '#ffc23c'],
	slate: ['#dde6ec', '#7d93a3'],
};

/** The two-dot face, frontend's --nomi-on-gradient-glow: dark on every tone's light gradient. */
export const FACE_COLOR = '#062019';
/** The face's eyes in the 48×48 view. */
export const FACE_EYES = [
	{ cx: 19, cy: 22, r: 2.6 },
	{ cx: 29, cy: 22, r: 2.6 },
] as const;

/** Who wears what: frontend's agentLook for each member's agent (Dana is money, Kala reminders…). */
export const CREW_LOOKS: Record<CrewId, AgentLook> = {
	nomi: { shape: 'cookie9', tone: 'glow', motion: 'spin' },
	koda: { shape: 'cookie6', tone: 'tide', motion: 'spin' },
	dana: { shape: 'sunny8', tone: 'ember', motion: 'spin' },
	kala: { shape: 'clover3', tone: 'citrus', motion: 'pulse' },
	rena: { shape: 'clover4', tone: 'sky', motion: 'spin' },
	tara: { shape: 'puffy7', tone: 'sky', motion: 'bounce' },
	maya: { shape: 'soft-square', tone: 'slate', motion: 'wobble' },
	raga: { shape: 'burst16', tone: 'bloom', motion: 'pulse' },
	personality: { shape: 'flower5', tone: 'bloom', motion: 'spin' },
	supervisor: { shape: 'sunny12', tone: 'dusk', motion: 'orbit' },
};
