import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ShareSection from './ShareSection.svelte';

describe('Share & export section', () => {
	it('shows the share and export panels, as the sheet draws them', async () => {
		await render(ShareSection);
		await expect.element(page.getByRole('heading', { name: 'Share', exact: true })).toBeInTheDocument();
		await expect.element(page.getByRole('heading', { name: 'Export warung-kopi-menu' })).toBeInTheDocument();
		expect(page.getByRole('dialog').elements()).toHaveLength(0);
	});
});
