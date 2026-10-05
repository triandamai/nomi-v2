import { describe, expect, it } from 'vitest';
import { fitPoints, strengthPips, timeAgo } from './memory';

describe('strengthPips', () => {
	it('puts a fresh memory in the middle and the clamps at the ends', () => {
		expect(strengthPips(1)).toBe(3);
		expect(strengthPips(5)).toBe(5);
		expect(strengthPips(0.1)).toBe(1);
		expect(strengthPips(50)).toBe(5);
	});
	it('moves with reinforcement', () => {
		expect(strengthPips(1 * 1.2 ** 4)).toBeGreaterThan(strengthPips(1));
		expect(strengthPips(1 * 0.8 ** 4)).toBeLessThan(strengthPips(1));
	});
});

describe('timeAgo', () => {
	const now = Date.parse('2026-10-05T12:00:00Z');
	it('reads naturally across ranges', () => {
		expect(timeAgo('2026-10-05T11:59:40Z', now)).toBe('just now');
		expect(timeAgo('2026-10-05T11:15:00Z', now)).toBe('45m ago');
		expect(timeAgo('2026-10-04T12:00:00Z', now)).toBe('1d ago');
		expect(timeAgo('2026-06-05T12:00:00Z', now)).toBe('4mo ago');
	});
});

describe('fitPoints', () => {
	it('fits points inside the padded box and centres a single point', () => {
		const fitted = fitPoints([{ x: -3, y: 0 }, { x: 3, y: 1 }, { x: 0, y: -2 }], 100, 10);
		for (const p of fitted) {
			expect(p.x).toBeGreaterThanOrEqual(10);
			expect(p.x).toBeLessThanOrEqual(90);
			expect(p.y).toBeGreaterThanOrEqual(10);
			expect(p.y).toBeLessThanOrEqual(90);
		}
		expect(fitPoints([{ x: 4, y: 4 }], 100, 10)).toEqual([{ x: 50, y: 50 }]);
	});
});
