import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import EstimateSection from './EstimateSection.svelte';

describe('Build estimate section', () => {
	it('shows the plan before it runs, then running and done', async () => {
		await render(EstimateSection);
		await expect.element(page.getByText('Menu and pre-order page for Warung Kopi Bu Siti')).toBeInTheDocument();
		await expect.element(page.getByText('Step 2 of 6 · menu page')).toBeInTheDocument();
		await expect.element(page.getByText('warung-kopi-menu-3f9a12.trian.space')).toBeInTheDocument();
	});
});
