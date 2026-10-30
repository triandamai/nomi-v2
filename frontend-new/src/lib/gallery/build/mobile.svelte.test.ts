import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import MobileBuildSection from './MobileBuildSection.svelte';

describe('Build on mobile section', () => {
	it('shows the project bar, ask bar, live app, plan, building and live, a domain record and together', async () => {
		await page.viewport(1512, 900);
		await render(MobileBuildSection);
		await expect.element(page.getByRole('radiogroup', { name: 'Chat or preview' })).toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: 'Ask Koda about this screen' })).toBeInTheDocument();
		await expect.element(page.getByText('Warung Kopi Bu Siti', { exact: true })).toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: 'Build it' })).toBeInTheDocument();
		await expect.element(page.getByText('Building warung-kopi-menu')).toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: 'Copy 76.76.21.21' })).toBeInTheDocument();
		await expect.element(page.getByText('Can we add a Makanan section? Nasi uduk and roti bakar, same pickup times.')).toBeInTheDocument();
		await expect.element(page.getByText("Sari's change is ready")).toBeInTheDocument();
	});

	it('draws every board 430pt wide, as the pen does', async () => {
		await page.viewport(1512, 900);
		const { container } = await render(MobileBuildSection);
		const widths = [...container.querySelectorAll('[data-board]')].map((board) => board.getBoundingClientRect().width);
		expect(widths).toEqual(Array(8).fill(430));
	});

	it('keeps a board title on one line and lets the note wrap, as the pen does', async () => {
		await page.viewport(1512, 900);
		await render(MobileBuildSection);
		await document.fonts.ready;
		expect(page.getByText('Teammate in chat', { exact: true }).element().getBoundingClientRect().height).toBe(17);
		const note = page.getByText('their colour, their name, who they asked').element().getBoundingClientRect();
		expect(note.top).toBe(page.getByText('Teammate in chat', { exact: true }).element().getBoundingClientRect().top);
		expect(note.height).toBe(28);
	});
});
