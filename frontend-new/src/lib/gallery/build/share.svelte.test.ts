import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ShareSection from './ShareSection.svelte';

describe('Share & export section', () => {
	it('shows both panels and opens them as dialogs', async () => {
		await render(ShareSection);
		await expect.element(page.getByRole('heading', { name: 'Export warung-kopi-menu' })).toBeInTheDocument();
		await page.getByRole('button', { name: 'Open export' }).click();
		await expect.element(page.getByRole('dialog', { name: 'Export warung-kopi-menu' })).toBeVisible();
	});
});
