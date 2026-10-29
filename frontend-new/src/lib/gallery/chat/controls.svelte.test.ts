import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ControlsSection from './ControlsSection.svelte';

describe('Message controls & composer section', () => {
	it('shows message actions, regenerate and copy, attachments, voice and limits, working', async () => {
		await render(ControlsSection);
		await expect.element(page.getByText('edited · 14:05')).toBeInTheDocument();
		await expect.element(page.getByRole('textbox', { name: 'Edit message' })).toBeInTheDocument();
		await page.getByRole('button', { name: 'Regenerate' }).click();
		await expect.element(page.getByRole('menuitem', { name: /More detail/ })).toBeVisible();
		await page.getByRole('menuitem', { name: /More detail/ }).click();
		await expect.element(page.getByRole('progressbar', { name: 'bca-oct-2026.pdf' })).toHaveAttribute('aria-valuenow', '62');
		await expect.element(page.getByRole('group', { name: 'Recording' })).toBeInTheDocument();
		await expect.element(page.getByText('12.480 / 10.000')).toBeInTheDocument();
	});
});
