import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { createRawSnippet } from 'svelte';
import { page } from 'vitest/browser';
import GallerySection from './GallerySection.svelte';
import { tokenColor } from '$lib/ds/test-utils';

// A design sheet's Section Head as the pen draws it (e.g. acyEz): a 28px title with the line
// beside it in 13px text-3, bottoms aligned.

describe('GallerySection', () => {
	it('heads a section like the pen', async () => {
		const children = createRawSnippet(() => ({ render: () => '<p>content</p>' }));
		await render(GallerySection, { title: 'Rich content', description: 'Structure only when the content has it.', children });
		const title = page.getByRole('heading', { name: 'Rich content' }).element();
		expect(getComputedStyle(title).fontSize).toBe('28px');
		expect(getComputedStyle(title).fontWeight).toBe('500');
		const line = page.getByText('Structure only when the content has it.').element();
		expect(getComputedStyle(line).fontSize).toBe('13px');
		expect(getComputedStyle(line).color).toBe(tokenColor('nm-text-3'));
		expect(getComputedStyle(title.parentElement!).alignItems).toBe('flex-end');
	});
});
