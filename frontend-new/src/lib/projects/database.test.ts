import { describe, expect, it } from 'vitest';
import { cellText, readTable } from './database';

describe('reading a table', () => {
	it('quotes the schema and name, and reads the first rows only', () => {
		expect(readTable({ schema: 'public', name: 'orders' }, 50)).toBe('SELECT * FROM "public"."orders" LIMIT 50');
		expect(readTable({ schema: 'my"schema', name: 'menu items' }, 5)).toBe('SELECT * FROM "my""schema"."menu items" LIMIT 5');
	});
});

describe('a cell', () => {
	it('says each value plainly', () => {
		expect([cellText(null), cellText('Rina'), cellText(48000), cellText(true), cellText({ a: 1 }), cellText(['x'])]).toEqual(['—', 'Rina', '48000', 'true', '{"a":1}', '["x"]']);
	});
});
