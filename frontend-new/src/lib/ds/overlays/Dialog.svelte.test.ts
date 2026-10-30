import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import DialogFixture from './DialogFixture.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Dialogs sit on nm-overlay, above everything, and keep focus inside until they close.

describe('Dialog', () => {
	it('opens above everything on the overlay, named by its title and described', async () => {
		await render(DialogFixture);
		await page.getByRole('button', { name: 'Delete project' }).click();
		const dialog = page.getByRole('dialog', { name: 'Delete salah-daily-tracker?' });
		await expect.element(dialog).toBeVisible();
		await expect.element(dialog).toHaveAccessibleDescription(/The live URL stops working/);
		const element = dialog.element() as HTMLDialogElement;
		expect(element.matches(':modal')).toBe(true);
		const style = getComputedStyle(element.firstElementChild!);
		expect(style.backgroundColor).toBe(tokenBackground('nm-surface'));
		expect(style.outlineColor).toBe(tokenColor('nm-border-strong'));
		expect(style.boxShadow).toContain('0px 8px 24px');
		expect(getComputedStyle(element, '::backdrop').backgroundColor).toBe(tokenBackground('nm-overlay'));
		await expect.element(page.getByRole('textbox', { name: 'Type the project name to confirm' })).toHaveFocus();
	});

	it('closes on Escape and on its own buttons', async () => {
		const onclose = vi.fn();
		await render(DialogFixture, { onclose });
		await page.getByRole('button', { name: 'Delete project' }).click();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
		expect(onclose).toHaveBeenCalledOnce();

		await page.getByRole('button', { name: 'Delete project' }).click();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
		expect(onclose).toHaveBeenCalledTimes(2);
	});
});
