// @vitest-environment jsdom
// The design-system gallery's sections: each renders its examples, and the examples' controls work.
import { describe, expect, it } from 'vitest';
import { clickEverything, render, settle } from '../../test-stubs/render';

const sections = import.meta.glob<{ default: unknown }>(['./*Section.svelte', '!./GallerySection.svelte', './*/*Section.svelte', './*/*/*Section.svelte', './*/WarungApp.svelte'], { eager: true });

describe('gallery', () => {
	it('has sections to show', () => {
		expect(Object.keys(sections).length).toBeGreaterThan(15);
	});

	for (const [name, module] of Object.entries(sections)) {
		it(`renders ${name}`, async () => {
			const root = render(module.default as never, {} as never);
			expect(root.innerHTML.length).toBeGreaterThan(0);
			clickEverything(root);
			await settle(3);
		});
	}
});
