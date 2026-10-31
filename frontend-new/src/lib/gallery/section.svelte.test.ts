import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { createRawSnippet } from 'svelte';
import { page } from 'vitest/browser';
import GallerySection from './GallerySection.svelte';
import GalleryBoard from './GalleryBoard.svelte';
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

	it('carries no padding of its own: the sheet spaces sections 64 apart, as the pen does', async () => {
		const children = createRawSnippet(() => ({ render: () => '<p>content</p>' }));
		const { container } = await render(GallerySection, { title: 'Story sheet', description: 'Values.', children });
		expect(getComputedStyle(container.querySelector('section')!).paddingTop).toBe('0px');
	});

	it('can be linked to: its id is its title, as the docs link to it', async () => {
		const children = createRawSnippet(() => ({ render: () => '<p>content</p>' }));
		const { container } = await render(GallerySection, { title: 'Code, terminal & phone', description: 'Values.', children });
		expect(container.querySelector('section')!.id).toBe('code-terminal-phone');
	});
});

describe('GalleryBoard', () => {
	it('centres the note on the title on the Nomi and Chat sheets (pen board heads, note 1.5px down)', async () => {
		const children = createRawSnippet(() => ({ render: () => '<p>content</p>' }));
		await render(GalleryBoard, { title: 'Buttons', note: 'primary · secondary', children });
		const title = page.getByText('Buttons', { exact: true }).element().getBoundingClientRect();
		expect(page.getByText('primary · secondary').element().getBoundingClientRect().top - title.top).toBe(1.5);
	});
});
