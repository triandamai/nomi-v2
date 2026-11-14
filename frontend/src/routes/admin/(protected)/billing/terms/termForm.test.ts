import { describe, expect, it } from 'vitest';
import { readTerm } from './termForm';

function form(entries: Record<string, string>) {
	const data = new FormData();
	for (const [key, value] of Object.entries(entries)) data.append(key, value);
	return data;
}

describe('readTerm', () => {
	it('reads a term as the backend takes it', () => {
		expect(readTerm(form({ months: '6', discount_percent: '10', label: ' Half year ', is_active: 'true', sort_order: '2' }))).toEqual({
			months: 6,
			discount_percent: 10,
			label: 'Half year',
			is_active: true,
			sort_order: 2,
		});
	});

	it('defaults the order to zero and hidden when not ticked', () => {
		expect(readTerm(form({ months: '1', discount_percent: '0', is_active: 'false' }))).toMatchObject({ sort_order: 0, is_active: false, label: '' });
	});

	it('refuses months below one and discounts outside 0 to 100', () => {
		expect(readTerm(form({ months: '0', discount_percent: '10' }))).toBeNull();
		expect(readTerm(form({ months: '3', discount_percent: '101' }))).toBeNull();
		expect(readTerm(form({ months: '3', discount_percent: '-1' }))).toBeNull();
		expect(readTerm(form({ months: '', discount_percent: '10' }))).toBeNull();
		expect(readTerm(form({ months: '2.5', discount_percent: '10' }))).toBeNull();
	});
});
