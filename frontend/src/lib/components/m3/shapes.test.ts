import { describe, expect, it } from 'vitest';
import { agentLook, shapePath, shapePoints, type ShapeName } from './shapes';

const SHAPES: ShapeName[] = ['cookie9', 'sunny8', 'cookie6', 'clover4', 'flower5', 'circle'];

describe('shapes', () => {
	it('samples every shape with the same point count so LoadingIndicator can morph between any pair', () => {
		const counts = new Set(SHAPES.map((name) => shapePoints(name).length));
		expect(counts.size).toBe(1);
	});

	it('keeps every shape inside the 48×48 viewBox', () => {
		for (const name of SHAPES) {
			for (const [x, y] of shapePoints(name)) {
				expect(x).toBeGreaterThanOrEqual(0);
				expect(x).toBeLessThanOrEqual(48);
				expect(y).toBeGreaterThanOrEqual(0);
				expect(y).toBeLessThanOrEqual(48);
			}
		}
	});

	it('builds a closed SVG path', () => {
		expect(shapePath('cookie9')).toMatch(/^M[\d.]+ [\d.]+( L[\d.]+ [\d.]+)+Z$/);
	});
});

describe('agentLook', () => {
	it('maps agent types and display names to the same identity', () => {
		expect(agentLook('money')).toEqual({ shape: 'sunny8', tone: 'ember' });
		expect(agentLook('Money')).toEqual({ shape: 'sunny8', tone: 'ember' });
		expect(agentLook('coding')).toEqual({ shape: 'cookie6', tone: 'tide' });
		expect(agentLook('Planning')).toEqual({ shape: 'clover4', tone: 'sky' });
		expect(agentLook('personality')).toEqual({ shape: 'flower5', tone: 'bloom' });
	});

	it('falls back to Nomi for chitchat, dynamic agents and missing names', () => {
		expect(agentLook('chitchat')).toEqual({ shape: 'cookie9', tone: 'glow' });
		expect(agentLook('Travel Scout')).toEqual({ shape: 'cookie9', tone: 'glow' });
		expect(agentLook(null)).toEqual({ shape: 'cookie9', tone: 'glow' });
	});
});
