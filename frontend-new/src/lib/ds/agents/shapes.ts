// The crew's agent shapes, as frontend draws them (frontend/src/lib/components/m3/shapes.ts):
// each member is a "polar" outline r(θ) = R·(1 + amplitude·cos(lobes·(θ − phase))) in a 48×48
// view, filled with their gradient. `motion` is how frontend moves each one while it works; Mascot
// doesn't take `working` yet, since frontend's twin would have to take it too. The pen's flat
// mascots are kept in $lib/rollback/PenMascot.svelte (with ./mascots.ts) to roll back to.

import type { AvatarDefinition } from 'open-bot-avatar';
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

// ── The live (interactive) mascot ────────────────────────────────────────────────────────────────
// open-bot-avatar (https://dominikscholz.github.io/open-bot-avatar/) turns a flat silhouette + an eye
// pair into a character with volume, blinking, gaze and moods. Its contract is one path in a
// 0 0 100 100 view (geometry inside 8..92) and one solid body colour, so the live mascot is the same
// outline scaled into that view, painted in the tone's middle colour (the engine adds its own shading).

/** open-bot-avatar's view side. */
const LIVE_VIEW = 100;
const LIVE_CENTER = LIVE_VIEW / 2;
// The biggest lobe (amplitude 0.2) reaches 34·1.2 = 40.8 from the centre: inside the 8..92 safe area.
const LIVE_RADIUS = 34;
const LIVE_SCALE = LIVE_RADIUS / RADIUS;
// Fewer points than the flat outline: the engine slices the path into many cross-sections.
const LIVE_POINTS = 72;

/** The outline of a shape as an SVG path in open-bot-avatar's 100×100 view. */
export function livePath(name: AgentShapeName): string {
	const { lobes, amplitude, phase = 0 } = SPECS[name];
	const points: string[] = [];
	for (let i = 0; i < LIVE_POINTS; i++) {
		const theta = (2 * Math.PI * i) / LIVE_POINTS;
		const r = LIVE_RADIUS * (1 + amplitude * Math.cos(lobes * (theta - phase)));
		points.push(`${(LIVE_CENTER + r * Math.cos(theta)).toFixed(DECIMALS)} ${(LIVE_CENTER + r * Math.sin(theta)).toFixed(DECIMALS)}`);
	}
	return `M${points.join(' L')}Z`;
}

/** Each tone as one solid colour (the middle of its gradient), for the live mascot. */
export const TONE_BODY: Record<AgentTone, string> = {
	glow: '#5be08f',
	ember: '#ffa462',
	tide: '#82d6d9',
	sky: '#a8ceff',
	bloom: '#ffb4c5',
	dusk: '#c1abff',
	citrus: '#ffdb70',
	slate: '#adbcc8',
};

/** The two-dot face placed as open-bot-avatar's eye pair: the flat face, scaled into its view. */
const LIVE_EYES = (() => {
	const [left, right] = FACE_EYES;
	return {
		x: LIVE_CENTER + ((left.cx + right.cx) / 2 - CENTER) * LIVE_SCALE,
		y: LIVE_CENTER + (left.cy - CENTER) * LIVE_SCALE,
		spacing: (right.cx - left.cx) * LIVE_SCALE,
		scale: 1,
	};
})();

/** A crew member's open-bot-avatar definition (contract v1). */
export function liveDefinition(agent: CrewId, name: string): AvatarDefinition {
	return {
		version: 1,
		id: agent,
		name,
		body: { path: livePath(CREW_LOOKS[agent].shape) },
		eyes: { ...LIVE_EYES },
	};
}

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
	iris: { shape: 'clover3', tone: 'dusk', motion: 'orbit' },
};
