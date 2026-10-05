// M3 Expressive shape library + the agent → (shape, gradient) identity mapping.
//
// Every shape is a "polar" outline r(θ) = R·(1 + amplitude·cos(lobes·θ)) sampled at the same
// number of points, all in a 48×48 viewBox. Sharing one point count is what lets
// MorphingShape morph between any two of them by plain per-point interpolation.

// Keep these three lists in sync with SHAPES, TONES and MOTIONS in
// backend/crates/nomi-server/src/routes/agents.rs (a dynamic agent stores its pick).
export const SHAPE_NAMES = [
	'cookie9',
	'sunny8',
	'cookie6',
	'clover4',
	'flower5',
	'circle',
	'sunny12',
	'clover3',
	'puffy7',
	'burst16',
	'wave10',
	'soft-square',
	'soft-triangle',
	'pentagon',
	'pill',
] as const;
export type ShapeName = (typeof SHAPE_NAMES)[number];

export const GRADIENT_TONES = ['glow', 'ember', 'tide', 'sky', 'bloom', 'dusk', 'citrus', 'slate'] as const;
export type GradientTone = (typeof GRADIENT_TONES)[number];

/** How a shape moves while its agent works (AgentShape's `working`). */
export const SHAPE_MOTIONS = ['spin', 'wobble', 'bounce', 'pulse', 'orbit'] as const;
export type ShapeMotion = (typeof SHAPE_MOTIONS)[number];

export const SHAPE_LABELS: Record<ShapeName, string> = {
	cookie9: 'Cookie',
	sunny8: 'Sunny',
	cookie6: 'Gear',
	clover4: 'Clover',
	flower5: 'Flower',
	circle: 'Circle',
	sunny12: 'Sun',
	clover3: 'Trefoil',
	puffy7: 'Puffy',
	burst16: 'Burst',
	wave10: 'Wave',
	'soft-square': 'Square',
	'soft-triangle': 'Triangle',
	pentagon: 'Pentagon',
	pill: 'Pill',
};

export const MOTION_LABELS: Record<ShapeMotion, string> = {
	spin: 'Spin',
	wobble: 'Wobble',
	bounce: 'Bounce',
	pulse: 'Pulse',
	orbit: 'Orbit',
};

// 120 keeps the outline smooth even at hero size (the auth stage draws one at 560px).
const POINTS = 120;
const RADIUS = 20;
const CENTER = 24;

// `phase` turns the shape (radians) so a flat side sits at the bottom where that reads better.
const SPECS: Record<ShapeName, { lobes: number; amplitude: number; phase?: number }> = {
	cookie9: { lobes: 9, amplitude: 0.09 },
	sunny8: { lobes: 8, amplitude: 0.06 },
	cookie6: { lobes: 6, amplitude: 0.1 },
	clover4: { lobes: 4, amplitude: 0.2 },
	flower5: { lobes: 5, amplitude: 0.16 },
	circle: { lobes: 1, amplitude: 0 },
	sunny12: { lobes: 12, amplitude: 0.045 },
	clover3: { lobes: 3, amplitude: 0.19, phase: -Math.PI / 2 },
	puffy7: { lobes: 7, amplitude: 0.13 },
	burst16: { lobes: 16, amplitude: 0.055 },
	wave10: { lobes: 10, amplitude: 0.08 },
	'soft-square': { lobes: 4, amplitude: 0.075, phase: Math.PI / 4 },
	'soft-triangle': { lobes: 3, amplitude: 0.12, phase: -Math.PI / 2 },
	pentagon: { lobes: 5, amplitude: 0.055, phase: -Math.PI / 2 },
	pill: { lobes: 2, amplitude: 0.17 },
};

export type Point = [number, number];

const pointCache = new Map<ShapeName, Point[]>();

export function shapePoints(name: ShapeName): Point[] {
	const cached = pointCache.get(name);
	if (cached) return cached;
	const { lobes, amplitude, phase = 0 } = SPECS[name];
	const points: Point[] = [];
	for (let i = 0; i < POINTS; i++) {
		const theta = (2 * Math.PI * i) / POINTS;
		const r = RADIUS * (1 + amplitude * Math.cos(lobes * (theta - phase)));
		points.push([CENTER + r * Math.cos(theta), CENTER + r * Math.sin(theta)]);
	}
	pointCache.set(name, points);
	return points;
}

export function pointsToPath(points: Point[]): string {
	return 'M' + points.map(([x, y]) => `${x.toFixed(2)} ${y.toFixed(2)}`).join(' L') + 'Z';
}

export function shapePath(name: ShapeName): string {
	return pointsToPath(shapePoints(name));
}

/** Gradient stops for each tone — the same values as the --nomi-gradient-* tokens in
 * material3.css, duplicated here because SVG fills can't read a CSS gradient. */
export const GRADIENT_STOPS: Record<GradientTone, string[]> = {
	glow: ['#e4f76a', '#5be08f', '#1db9a0'],
	ember: ['#ffd27a', '#ff7a4a'],
	tide: ['#c4f3ea', '#3fb8c8'],
	sky: ['#d6ecff', '#7ab0ff'],
	bloom: ['#ffd9e2', '#ff8fa8'],
	dusk: ['#e6dcff', '#9b7bff'],
	citrus: ['#fff4a3', '#ffc23c'],
	slate: ['#dde6ec', '#7d93a3'],
};

/** A deep step of each tone for small marks (checks, dots) that need more contrast than a
 * gradient's light end gives. */
export const TONE_ACCENT: Record<GradientTone, string> = {
	glow: '#0b6b4a',
	ember: '#b8430f',
	tide: '#0e7490',
	sky: '#3e7be0',
	bloom: '#be185d',
	dusk: '#6d4fd8',
	citrus: '#a16207',
	slate: '#475569',
};

export interface AgentLook {
	shape: ShapeName;
	tone: GradientTone;
	motion: ShapeMotion;
}

const NOMI_LOOK: AgentLook = { shape: 'cookie9', tone: 'glow', motion: 'spin' };

// Dynamic agents' chosen looks, keyed by lowercase agent_type and name (see registerAgentLooks).
const registered = new Map<string, AgentLook>();

export function isShapeName(value: unknown): value is ShapeName {
	return typeof value === 'string' && (SHAPE_NAMES as readonly string[]).includes(value);
}
export function isGradientTone(value: unknown): value is GradientTone {
	return typeof value === 'string' && (GRADIENT_TONES as readonly string[]).includes(value);
}
export function isShapeMotion(value: unknown): value is ShapeMotion {
	return typeof value === 'string' && (SHAPE_MOTIONS as readonly string[]).includes(value);
}

/** Teaches agentLook the crew's dynamic agents (the app layout calls this with /api/agents). */
export function registerAgentLooks(
	members: { agent_type: string; name: string; shape?: string | null; tone?: string | null; motion?: string | null }[],
): void {
	for (const member of members) {
		if (!isShapeName(member.shape)) continue;
		const look: AgentLook = {
			shape: member.shape,
			tone: isGradientTone(member.tone) ? member.tone : 'glow',
			motion: isShapeMotion(member.motion) ? member.motion : 'spin',
		};
		registered.set(member.agent_type.toLowerCase(), look);
		registered.set(member.name.toLowerCase(), look);
	}
}

/** Which shape + gradient + motion an agent wears. Accepts either an agent_type ("money") or a
 * display name ("Money"); a registered dynamic agent wears its own pick, and anything else
 * unrecognized — chitchat included — is Nomi. */
export function agentLook(agent: string | null | undefined): AgentLook {
	const key = (agent ?? '').toLowerCase();
	const custom = registered.get(key);
	if (custom) return custom;
	if (key.includes('money') || key.includes('budget')) return { shape: 'sunny8', tone: 'ember', motion: 'spin' };
	if (key.includes('cod')) return { shape: 'cookie6', tone: 'tide', motion: 'spin' };
	if (key.includes('plan')) return { shape: 'clover4', tone: 'sky', motion: 'spin' };
	if (key.includes('personality') || key.includes('memory')) return { shape: 'flower5', tone: 'bloom', motion: 'spin' };
	if (key.includes('supervisor')) return { shape: 'sunny12', tone: 'dusk', motion: 'orbit' };
	return NOMI_LOOK;
}
