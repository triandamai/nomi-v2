import { describe, expect, it } from 'vitest';
import { tableListColumns, tableListItem } from './tableList';

describe('tableList', () => {
	it('reads a transaction table as name, amount, category and date', () => {
		const columns = [
			{ key: 'date', label: 'Date' },
			{ key: 'merchant', label: 'Merchant' },
			{ key: 'category', label: 'Category' },
			{ key: 'amount', label: 'Amount' },
		];
		const rows = [
			{ date: '2026-10-02', merchant: 'Kopi Kenangan', category: 'Food', amount: 'Rp 35.000' },
			{ date: '2026-10-03', merchant: 'Gojek', category: 'Transport', amount: 'Rp 22.500' },
		];
		const cols = tableListColumns(columns, rows);
		expect(tableListItem(rows[0], cols)).toEqual({
			headline: 'Kopi Kenangan',
			trailing: 'Rp 35.000',
			category: 'Food',
			supporting: ['2026-10-02'],
		});
	});

	it('uses the last numeric column as the trailing value when none is named like an amount', () => {
		const columns = [
			{ key: 'item', label: 'Item' },
			{ key: 'qty', label: 'Qty' },
		];
		const rows = [{ item: 'Eggs', qty: 12 }];
		const item = tableListItem(rows[0], tableListColumns(columns, rows));
		expect(item.headline).toBe('Eggs');
		expect(item.trailing).toBe('12');
	});

	it('keeps a text-only table readable', () => {
		const columns = [
			{ key: 'name', label: 'Name' },
			{ key: 'note', label: 'Note' },
		];
		const rows = [{ name: 'Bali', note: 'Beach days' }];
		expect(tableListItem(rows[0], tableListColumns(columns, rows))).toEqual({
			headline: 'Bali',
			trailing: null,
			category: null,
			supporting: ['Beach days'],
		});
	});
});
