import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import OverlaysSection from './OverlaysSection.svelte';

// As the pen draws the Overlays row: the menu open with its item states, the toasts and a tooltip,
// and a dialog on the scrim.
describe('Overlays section', () => {
	it('shows the menu open, its three item states, the toasts, a tooltip and the dialog', async () => {
		await render(OverlaysSection);
		const menu = page.getByRole('menu', { name: 'Project actions' });
		expect(menu.getByRole('menuitem').elements()).toHaveLength(4);
		for (const state of ['Menu Item / Default', 'Menu Item / Hover', 'Menu Item / Danger']) {
			await expect.element(page.getByRole('menu', { name: state })).toBeInTheDocument();
		}
		expect(page.getByRole('alert').elements()).toHaveLength(1);
		expect(page.getByRole('status').elements()).toHaveLength(2);
		await expect.element(page.getByRole('tooltip')).toHaveTextContent(/New chat\s*⌘K/);
		await expect.element(page.getByRole('heading', { name: 'Delete salah-daily-tracker?' })).toBeInTheDocument();
	});

	it('gives the dialog’s Delete project button the trash icon', async () => {
		await render(OverlaysSection);
		const remove = page.getByRole('button', { name: 'Delete project' }).element();
		expect(remove.querySelector('svg')?.classList).toContain('lucide-trash-2');
	});
});
