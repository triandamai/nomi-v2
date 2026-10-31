import { describe, expect, it } from 'vitest';
import { CATEGORIES, catalog } from './catalog';

describe('catalog', () => {
	const items = catalog(['inputs/Composer.svelte', 'actions/Button.svelte', 'mobile/Composer.svelte', 'conversation/KVRow.svelte', 'actions/IconButton.svelte']);

	it('orders components by category, then name', () => {
		expect(items.map((item) => item.file)).toEqual(['actions/Button.svelte', 'actions/IconButton.svelte', 'inputs/Composer.svelte', 'conversation/KVRow.svelte', 'mobile/Composer.svelte']);
	});

	it('names each page after the component, and tells same-named ones apart by their folder', () => {
		expect(items.map((item) => [item.name, item.slug])).toEqual([
			['Button', 'button'],
			['IconButton', 'icon-button'],
			['Composer', 'inputs-composer'],
			['KVRow', 'kv-row'],
			['Composer', 'mobile-composer'],
		]);
	});

	it('knows every folder of the design system', () => {
		expect(items.every((item) => CATEGORIES.some((category) => category.key === item.category))).toBe(true);
		expect(() => catalog(['nowhere/Thing.svelte'])).toThrow(/nowhere/);
	});
});
