// M3 Expressive shape library + the agent → (shape, gradient) identity mapping.
//
// Every shape is a "polar" outline r(θ) = R·(1 + amplitude·cos(lobes·θ)) sampled at the same
// number of points, all in a 48×48 viewBox. Sharing one point count is what lets
// MorphingShape morph between any two of them by plain per-point interpolation.

export type ShapeName = 'cookie9' | 'sunny8' | 'cookie6' | 'clover4' | 'flower5' | 'circle';
export type GradientTone = 'glow' | 'ember' | 'tide' | 'sky' | 'bloom';

// 120 keeps the outline smooth even at hero size (the auth stage draws one at 560px).
const POINTS = 120;
const RADIUS = 20;
const CENTER = 24;

const SPECS: Record<ShapeName, { lobes: number; amplitude: number }> = {
	cookie9: { lobes: 9, amplitude: 0.09 },
	sunny8: { lobes: 8, amplitude: 0.06 },
	cookie6: { lobes: 6, amplitude: 0.1 },
	clover4: { lobes: 4, amplitude: 0.2 },
	flower5: { lobes: 5, amplitude: 0.16 },
	circle: { lobes: 1, amplitude: 0 },
};

export type Point = [number, number];

const pointCache = new Map<ShapeName, Point[]>();

export function shapePoints(name: ShapeName): Point[] {
	const cached = pointCache.get(name);
	if (cached) return cached;
	const { lobes, amplitude } = SPECS[name];
	const points: Point[] = [];
	for (let i = 0; i < POINTS; i++) {
		const theta = (2 * Math.PI * i) / POINTS;
		const r = RADIUS * (1 + amplitude * Math.cos(lobes * theta));
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
};

export interface AgentLook {
	shape: ShapeName;
	tone: GradientTone;
}

/** Which shape + gradient an agent wears. Accepts either an agent_type ("money") or a display
 * name ("Money"); anything unrecognized — including chitchat and dynamic agents — is Nomi. */
export function agentLook(agent: string | null | undefined): AgentLook {
	const key = (agent ?? '').toLowerCase();
	if (key.includes('money') || key.includes('budget')) return { shape: 'sunny8', tone: 'ember' };
	if (key.includes('cod')) return { shape: 'cookie6', tone: 'tide' };
	if (key.includes('plan')) return { shape: 'clover4', tone: 'sky' };
	if (key.includes('personality') || key.includes('memory')) return { shape: 'flower5', tone: 'bloom' };
	return { shape: 'cookie9', tone: 'glow' };
}
