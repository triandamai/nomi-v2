import { describe, expect, it } from 'vitest';
import { nextIndex, placePanel } from './floating';

const viewport = { width: 1512, height: 862 };

describe('placePanel', () => {
	it('opens below its button, its start edge on the button’s', () => {
		expect(placePanel({ top: 252, bottom: 272, left: 150, right: 230 }, { width: 360, height: 356 }, viewport)).toEqual({ top: 276, left: 150 });
	});

	it('opens above when there’s no room below (the account menu off the user row)', () => {
		expect(placePanel({ top: 810, bottom: 846, left: 12, right: 236 }, { width: 272, height: 382 }, viewport)).toEqual({ top: 424, left: 12 });
	});

	it('lines up its end edge with the button’s when asked', () => {
		expect(placePanel({ top: 10, bottom: 40, left: 1300, right: 1400 }, { width: 236, height: 200 }, viewport, 'end')).toEqual({ top: 44, left: 1164 });
	});

	it('stays 8px inside the window', () => {
		expect(placePanel({ top: 10, bottom: 40, left: 300, right: 380 }, { width: 360, height: 200 }, { width: 390, height: 844 })).toEqual({ top: 44, left: 22 });
		expect(placePanel({ top: 300, bottom: 330, left: 2, right: 40 }, { width: 200, height: 600 }, { width: 390, height: 620 })).toEqual({ top: 8, left: 8 });
	});
});

describe('nextIndex', () => {
	it('moves through a menu’s items as the platform does, wrapping around', () => {
		expect(nextIndex('ArrowDown', 0, 4)).toBe(1);
		expect(nextIndex('ArrowDown', 3, 4)).toBe(0);
		expect(nextIndex('ArrowUp', 0, 4)).toBe(3);
		expect(nextIndex('Home', 2, 4)).toBe(0);
		expect(nextIndex('End', 0, 4)).toBe(3);
		// Nothing focused yet: down starts at the top, up at the bottom.
		expect(nextIndex('ArrowDown', -1, 4)).toBe(0);
		expect(nextIndex('ArrowUp', -1, 4)).toBe(3);
	});

	it('leaves other keys alone', () => {
		expect(nextIndex('Enter', 1, 4)).toBeNull();
		expect(nextIndex('ArrowDown', 0, 0)).toBeNull();
	});
});
