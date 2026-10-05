import { describe, expect, it } from 'vitest';
import { agentLook, registerAgentLooks, SHAPE_NAMES, shapePath, shapePoints } from './shapes';

const SHAPES = SHAPE_NAMES;

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
		expect(agentLook('money')).toMatchObject({ shape: 'sunny8', tone: 'ember' });
		expect(agentLook('Money')).toMatchObject({ shape: 'sunny8', tone: 'ember' });
		expect(agentLook('coding')).toMatchObject({ shape: 'cookie6', tone: 'tide' });
		expect(agentLook('Planning')).toMatchObject({ shape: 'clover4', tone: 'sky' });
		expect(agentLook('personality')).toMatchObject({ shape: 'flower5', tone: 'bloom' });
		expect(agentLook('Supervisor')).toEqual({ shape: 'sunny12', tone: 'dusk', motion: 'orbit' });
	});

	it('falls back to Nomi for chitchat, unknown agents and missing names', () => {
		expect(agentLook('chitchat')).toEqual({ shape: 'cookie9', tone: 'glow', motion: 'spin' });
		expect(agentLook('Someone New')).toEqual({ shape: 'cookie9', tone: 'glow', motion: 'spin' });
		expect(agentLook(null)).toEqual({ shape: 'cookie9', tone: 'glow', motion: 'spin' });
	});

	it('gives a registered dynamic agent its own look by type or by name', () => {
		registerAgentLooks([{ agent_type: 'b2f0-dyn', name: 'Travel Scout', shape: 'puffy7', tone: 'citrus', motion: 'bounce' }]);
		const look = { shape: 'puffy7', tone: 'citrus', motion: 'bounce' };
		expect(agentLook('b2f0-dyn')).toEqual(look);
		expect(agentLook('Travel Scout')).toEqual(look);
	});

	it('ignores a registered look it does not know', () => {
		registerAgentLooks([{ agent_type: 'odd', name: 'Odd One', shape: 'dodecahedron', tone: 'glow', motion: 'spin' }]);
		expect(agentLook('Odd One').shape).toBe('cookie9');
	});
});
