import type { TableColumn } from '$lib/types';

/** How a table row reads as a list item on a narrow screen. */
export interface TableListLayout {
	/** The row's name: what it is (merchant, item, title). */
	headline: string;
	/** The number to the right (amount, total, price), if the table has one. */
	trailing: string | null;
	/** A category column, shown as its shape before the text. */
	category: string | null;
	/** The remaining columns, kept in a list for the line below the headline. */
	supporting: string[];
}

const AMOUNT = /amount|total|price|cost|spent|sum|balance|value|jumlah|nominal|harga|biaya/i;
const CATEGORY = /^(category|kategori|type)$/i;
const NUMBER = /^[^\d-]{0,4}-?[\d.,\s]+[^\d]{0,4}$/;
const DATE = /^\d{4}-\d{2}-\d{2}|^\d{1,2}[/ -]\w{1,9}[/ -]\d{2,4}|^(mon|tue|wed|thu|fri|sat|sun)/i;

function text(value: unknown): string {
	return value === null || value === undefined ? '' : String(value).trim();
}

function mostly(rows: Record<string, unknown>[], key: string, test: (v: string) => boolean): boolean {
	const values = rows.map((r) => text(r[key])).filter(Boolean);
	return values.length > 0 && values.filter(test).length / values.length >= 0.6;
}

/** Picks which column plays which part, once per table. */
export function tableListColumns(columns: TableColumn[], rows: Record<string, unknown>[]) {
	const label = (c: TableColumn) => `${c.key} ${c.label}`;
	const numeric = (c: TableColumn) => mostly(rows, c.key, (v) => NUMBER.test(v));
	const trailing =
		columns.find((c) => AMOUNT.test(label(c))) ?? [...columns].reverse().find((c) => numeric(c) && !mostly(rows, c.key, (v) => DATE.test(v))) ?? null;
	const category = columns.find((c) => c !== trailing && (CATEGORY.test(c.key) || CATEGORY.test(c.label))) ?? null;
	const rest = columns.filter((c) => c !== trailing && c !== category);
	const headline = rest.find((c) => !numeric(c) && !mostly(rows, c.key, (v) => DATE.test(v))) ?? rest[0] ?? trailing ?? columns[0];
	return { headline, trailing: trailing === headline ? null : trailing, category, supporting: rest.filter((c) => c !== headline) };
}

export function tableListItem(row: Record<string, unknown>, cols: ReturnType<typeof tableListColumns>): TableListLayout {
	return {
		headline: text(row[cols.headline?.key ?? '']),
		trailing: cols.trailing ? text(row[cols.trailing.key]) || null : null,
		category: cols.category ? text(row[cols.category.key]) || null : null,
		supporting: cols.supporting.map((c) => text(row[c.key])).filter(Boolean),
	};
}
