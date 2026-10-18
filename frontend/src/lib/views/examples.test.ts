import { describe, expect, it } from 'vitest';
import examples from './examples.json';
import { VIEW_NAMES } from './names';
import { chartScale, composeForm, formatValue, formatWhen } from './format';
import type { View } from './types';

describe('views', () => {
	it('draws every view the backend has an example of, and has an example for every view it draws', () => {
		const names = (examples as View[]).map((e) => e.view);
		expect(new Set(names)).toEqual(new Set(VIEW_NAMES));
		expect(names.length).toBe(VIEW_NAMES.length);
	});

	it('formats values by their format', () => {
		expect(formatValue(45000, 'money', 'en')).toBe('45,000.00');
		expect(formatValue('12.5', 'money', 'en')).toBe('12.50');
		expect(formatValue(0.4, 'percent', 'en')).toBe('40%');
		expect(formatValue(40, 'percent', 'en')).toBe('40%');
		expect(formatValue(1234567, 'number', 'en')).toBe('1,234,567');
		expect(formatValue('2026-10-12', 'date', 'en')).toBe('Oct 12, 2026');
		expect(formatValue('not a date', 'date', 'en')).toBe('not a date');
		expect(formatValue(true, undefined, 'en')).toBe('✓');
		expect(formatValue('Rp 45.000', 'money', 'en')).toBe('Rp 45.000');
	});

	it('reads a timeline time as a day and a time, or keeps a label', () => {
		expect(formatWhen('Day 2', 'en')).toEqual({ day: 'Day 2', time: null });
		expect(formatWhen('2026-10-24', 'en').time).toBeNull();
		expect(formatWhen('2026-10-24T08:00:00', 'en').time).toMatch(/08:00/);
	});

	it('rounds a chart axis up to round steps', () => {
		expect(chartScale(1_500_000)).toEqual([0, 500_000, 1_000_000, 1_500_000]);
		expect(chartScale(7)).toEqual([0, 2, 4, 6, 8]);
		expect(chartScale(0)).toEqual([0, 1]);
	});

	it('writes a form as one message, leaving out empty fields', () => {
		const fields = [
			{ key: 'to', label: 'Where to' },
			{ key: 'notes', label: 'Notes' },
		];
		expect(composeForm('Trip', fields, { to: 'Bandung', notes: ' ' })).toBe('Trip\nWhere to: Bandung');
		expect(composeForm(undefined, fields, { to: '', notes: '' })).toBe('');
	});
});
