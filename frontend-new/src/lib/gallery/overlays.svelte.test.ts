import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import OverlaysSection from './OverlaysSection.svelte';

describe('Overlays section', () => {
	it('opens each overlay for real and shows the three toasts', async () => {
		await render(OverlaysSection);
		await page.getByRole('button', { name: 'Project actions' }).click();
		await expect.element(page.getByRole('menu')).toBeVisible();
		await page.getByRole('menuitem', { name: /Export repo/ }).click();
		await page.getByRole('button', { name: 'Delete project' }).first().click();
		await expect.element(page.getByRole('dialog', { name: 'Delete salah-daily-tracker?' })).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		expect(page.getByRole('alert').elements()).toHaveLength(1);
		expect(page.getByRole('status').elements().length).toBeGreaterThanOrEqual(2);
	});

	it('gives the dialog’s Delete project button the trash icon', async () => {
		await render(OverlaysSection);
		await page.getByRole('button', { name: 'Delete project' }).first().click();
		const dialog = page.getByRole('dialog', { name: 'Delete salah-daily-tracker?' });
		const remove = dialog.getByRole('button', { name: 'Delete project' }).element();
		expect(remove.querySelector('svg')?.classList).toContain('lucide-trash-2');
		await page.getByRole('button', { name: 'Cancel' }).click();
	});
});
