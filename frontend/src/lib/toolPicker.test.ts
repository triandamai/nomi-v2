import { describe, expect, it } from 'vitest';
import { m } from '$lib/paraglide/messages';
import { addTool, catalogFrom, filterTools, removeLastTool, removeTool, toolDescription } from './toolPicker';
import type { CatalogTool } from './types';

const tool = (name: string, description: string, keywords: string[] = [], group = 'Money'): CatalogTool => ({ name, description, keywords, group });
const catalog = [
	tool('list_transactions', 'List recent transactions', ['spending', 'expenses']),
	tool('log_transaction', 'Record an expense', ['expense']),
	tool('read_file', 'Read a project file', ['open'], 'Coding'),
	tool('transfer_notes', 'Move notes around', []),
];

describe('filterTools', () => {
	it('keeps the whole catalog for an empty query', () => {
		expect(filterTools(catalog, '  ')).toBe(catalog);
	});

	it('ranks name prefix, then name contains, then keyword, then description', () => {
		expect(filterTools(catalog, 'trans').map((t) => t.name)).toEqual(['transfer_notes', 'list_transactions', 'log_transaction']);
		expect(filterTools(catalog, 'expense').map((t) => t.name)).toEqual(['list_transactions', 'log_transaction']);
		expect(filterTools(catalog, 'project').map((t) => t.name)).toEqual(['read_file']);
	});

	it('is case-insensitive and lets spaces stand for underscores', () => {
		expect(filterTools(catalog, 'READ FILE').map((t) => t.name)).toEqual(['read_file']);
	});

	it('searches the description that is shown', () => {
		expect(filterTools(catalog, 'baca', (t) => (t.name === 'read_file' ? 'Baca file proyek' : t.description)).map((t) => t.name)).toEqual(['read_file']);
	});

	it('returns nothing when nothing matches', () => {
		expect(filterTools(catalog, 'zzz')).toEqual([]);
	});
});

describe('selection', () => {
	it('adds once, removes and drops the last', () => {
		expect(addTool(['a'], 'b')).toEqual(['a', 'b']);
		expect(addTool(['a'], 'a')).toEqual(['a']);
		expect(removeTool(['a', 'b'], 'a')).toEqual(['b']);
		expect(removeLastTool(['a', 'b'])).toEqual(['a']);
		expect(removeLastTool([])).toEqual([]);
	});
});

describe('toolDescription', () => {
	it('uses the translated message when there is one, else the catalog text', () => {
		expect(toolDescription(tool('read_file', 'API text'))).toBe(m.tooldesc_read_file());
		expect(toolDescription(tool('brand_new_tool', 'API text'))).toBe('API text');
	});
});

describe('catalogFrom', () => {
	it('reads the tools of the reply and tolerates anything else', () => {
		expect(catalogFrom({ tools: catalog })).toBe(catalog);
		expect(catalogFrom([])).toEqual([]);
		expect(catalogFrom(null)).toEqual([]);
	});
});
