import { describe, expect, it } from 'vitest';
import { projectTo3D } from './pca';

describe('projectTo3D', () => {
	it('returns one 3D point per input vector', () => {
		const points = projectTo3D([
			[1, 0, 0, 0],
			[0, 1, 0, 0],
			[0, 0, 1, 0],
			[1, 1, 1, 1],
		]);
		expect(points).toHaveLength(4);
		for (const p of points) {
			expect(Number.isFinite(p.x)).toBe(true);
			expect(Number.isFinite(p.y)).toBe(true);
			expect(Number.isFinite(p.z)).toBe(true);
		}
	});

	it('places two identical vectors at the same point', () => {
		const points = projectTo3D([
			[1, 2, 3, 4, 5],
			[1, 2, 3, 4, 5],
			[9, -3, 0, 5, 2],
			[4, 4, 4, 4, 4],
		]);
		expect(points[0].x).toBeCloseTo(points[1].x, 5);
		expect(points[0].y).toBeCloseTo(points[1].y, 5);
		expect(points[0].z).toBeCloseTo(points[1].z, 5);
	});

	it('separates two very different vectors along the dominant axis', () => {
		const points = projectTo3D([
			[10, 0, 0],
			[10, 0, 0],
			[-10, 0, 0],
			[-10, 0, 0],
		]);
		expect(Math.abs(points[0].x - points[2].x)).toBeGreaterThan(1);
	});

	it('handles fewer than two vectors without throwing', () => {
		expect(projectTo3D([])).toEqual([]);
		expect(projectTo3D([[1, 2, 3]])).toEqual([{ x: 0, y: 0, z: 0 }]);
	});
});
