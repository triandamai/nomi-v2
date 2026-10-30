import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import MenuFixture from './MenuFixture.svelte';
import { tokenColor } from '../test-utils';

// A floating layer: surface fill, border-strong, the one shadow. Keyboard works like the
// platform's menus: arrows move, Enter picks, Escape closes and goes back to the button.

describe('Menu', () => {
	it('opens from its button as a floating list of items', async () => {
		await render(MenuFixture, { onselect: () => {} });
		const button = page.getByRole('button', { name: 'Project actions' });
		await expect.element(button).toHaveAttribute('aria-haspopup', 'menu');
		await expect.element(button).toHaveAttribute('aria-expanded', 'false');
		await button.click();
		await expect.element(button).toHaveAttribute('aria-expanded', 'true');

		const menu = page.getByRole('menu');
		await expect.element(menu).toBeVisible();
		expect(page.getByRole('menuitem').elements()).toHaveLength(4);
		const style = getComputedStyle(menu.element());
		expect(style.outlineColor).toBe(tokenColor('nm-border-strong'));
		expect(style.outlineOffset).toBe('-1px');
		expect(style.borderTopLeftRadius).toBe('10px');
		expect(style.boxShadow).toContain('0px 8px 24px');
		expect(getComputedStyle(page.getByText('salah-daily-tracker').element()).textTransform).toBe('uppercase');
		expect(getComputedStyle(page.getByText('⌘E').element()).fontFamily).toContain('Geist Mono');
		expect(getComputedStyle(page.getByRole('menuitem', { name: 'Delete project' }).element()).color).toBe(tokenColor('nm-danger'));
	});

	it('is driven by the keyboard', async () => {
		const onselect = vi.fn();
		await render(MenuFixture, { onselect });
		await page.getByRole('button', { name: 'Project actions' }).click();
		await expect.element(page.getByRole('menuitem', { name: /Export repo/ })).toHaveFocus();
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(page.getByRole('menuitem', { name: /Visit live site/ })).toHaveFocus();
		await userEvent.keyboard('{End}');
		await expect.element(page.getByRole('menuitem', { name: /Delete project/ })).toHaveFocus();
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(page.getByRole('menuitem', { name: /Export repo/ })).toHaveFocus();
		await userEvent.keyboard('{ArrowUp}{Enter}');
		expect(onselect).toHaveBeenCalledWith('delete');
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: 'Project actions' })).toHaveFocus();
	});

	it('closes on Escape or a click elsewhere, choosing nothing', async () => {
		const onselect = vi.fn();
		await render(MenuFixture, { onselect });
		const button = page.getByRole('button', { name: 'Project actions' });
		await button.click();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
		await expect.element(button).toHaveFocus();

		await button.click();
		await page.getByRole('button', { name: 'Elsewhere' }).click();
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
		expect(onselect).not.toHaveBeenCalled();
	});
});
