import { describe, expect, it } from 'vitest';
import { DEFAULT_HOME_LAYOUT, HOME_LAYOUTS, homeLayout, isHomeLayoutId, layoutBlocks } from './homeLayouts';

describe('home layouts', () => {
	it('falls back to the default for a missing or unknown slug', () => {
		expect(homeLayout(null).id).toBe(DEFAULT_HOME_LAYOUT);
		expect(homeLayout('from-a-newer-build').id).toBe(DEFAULT_HOME_LAYOUT);
		expect(homeLayout('stage').id).toBe('stage');
	});

	it('knows its own ids only', () => {
		expect(isHomeLayoutId('classic')).toBe(true);
		expect(isHomeLayoutId('Stage')).toBe(false);
		expect(isHomeLayoutId(undefined)).toBe(false);
	});

	it('gives every layout a unique slug the backend accepts', () => {
		const ids = HOME_LAYOUTS.map((layout) => layout.id);
		expect(new Set(ids).size).toBe(ids.length);
		for (const id of ids) expect(id).toMatch(/^[a-z][a-z0-9-]{0,31}$/);
	});

	it('keeps a composer on every home page and ratios matching their columns', () => {
		for (const layout of HOME_LAYOUTS) {
			expect(layoutBlocks(layout).has('composer'), layout.id).toBe(true);
			for (const row of layout.rows) {
				if (row.ratio) expect(row.ratio.length, layout.id).toBe(row.columns.length);
			}
		}
	});

	it('lists the blocks a layout uses', () => {
		expect([...layoutBlocks(homeLayout('stage'))].sort()).toEqual(['composer', 'crew', 'greeting', 'recent', 'status']);
	});
});
