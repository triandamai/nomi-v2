import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import StorySection from './StorySection.svelte';
import TypesSection from './TypesSection.svelte';

// The Build sheet's last two sections (pen p6ZMJc, OFwmY): the one story every frame's numbers
// come from, and what changes per type of build. Sizes are the pen's.

describe('Story sheet section', () => {
	it('lists the story values, keys in text-2 and values in mono, 41pt a row', async () => {
		await page.viewport(1512, 900);
		const { container } = await render(StorySection);
		const sheet = container.querySelector('dl')!;
		expect(sheet.querySelectorAll('dt').length).toBe(12);
		expect(getComputedStyle(page.getByText('Mario Prasetyo').element()).fontFamily).toContain('Geist Mono');
		expect(sheet.getBoundingClientRect().height).toBe(492);
		expect(page.getByText('Run pressed / build started').element().getBoundingClientRect().width).toBe(220);
	});
});

describe('Build types section', () => {
	it('is a table of five types by what Koda sets up, the tabs, share and monitor', async () => {
		await page.viewport(1512, 900);
		await render(TypesSection);
		const table = page.getByRole('table', { name: 'Build types' });
		expect(table.getByRole('columnheader').elements().map((cell) => cell.textContent?.trim())).toEqual([
			'Type',
			'Koda sets up',
			'Workspace tabs',
			'Share leads with',
			'Monitor leads with',
		]);
		expect(table.getByRole('rowheader').elements().length).toBe(5);
		expect(table.element().getBoundingClientRect().height).toBe(255);
		await expect.element(page.getByText(/^Rules: the Database tab and dump only exist/)).toBeVisible();
	});
});
