import { describe, expect, it } from 'vitest';
import { DEFAULT_PINS, moveItem, resolvePins } from './drawerPins';

describe('resolvePins', () => {
	it('falls back to the default drawer until the person customises it', () => {
		expect(resolvePins(null)).toEqual(DEFAULT_PINS);
		expect(resolvePins([])).toEqual([]);
	});

	it('keeps the saved order and drops unknown or repeated entries', () => {
		expect(resolvePins(['crew', 'money', 'home', 'crew', 'nope'])).toEqual(['crew', 'money']);
	});
});

describe('moveItem', () => {
	it('moves an item up or down', () => {
		expect(moveItem(['a', 'b', 'c', 'd'], 0, 2)).toEqual(['b', 'c', 'a', 'd']);
		expect(moveItem(['a', 'b', 'c', 'd'], 3, 0)).toEqual(['d', 'a', 'b', 'c']);
		expect(moveItem(['a', 'b'], 1, 9)).toEqual(['a', 'b']);
	});
});
