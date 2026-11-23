import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ThemePickerButton from './ThemePickerButton.svelte';
import { setThemeContext } from './themeContext';
import ContextFixture from './ThemePickerContextFixture.svelte';

afterEach(() => {
	document.body.innerHTML = '';
});

describe('ThemePickerButton', () => {
	it('renders an icon button with Theme label', async () => {
		await render(ThemePickerButton, { theme: 'system' });
		const button = page.getByRole('button', { name: 'Theme' });
		await expect.element(button).toBeVisible();
	});

	it('opens bottom sheet with system, light, and dark options on click', async () => {
		const ontheme = vi.fn();
		await render(ThemePickerButton, { theme: 'system', ontheme });

		const button = page.getByRole('button', { name: 'Theme' });
		await button.click();

		const dialog = page.getByRole('dialog', { name: 'Theme' });
		await expect.element(dialog).toBeVisible();

		const systemOption = page.getByRole('button', { name: 'System' });
		const lightOption = page.getByRole('button', { name: 'Light' });
		const darkOption = page.getByRole('button', { name: 'Dark' });

		await expect.element(systemOption).toBeVisible();
		await expect.element(lightOption).toBeVisible();
		await expect.element(darkOption).toBeVisible();

		await expect.element(systemOption).toHaveAttribute('aria-pressed', 'true');
		await expect.element(lightOption).toHaveAttribute('aria-pressed', 'false');
		await expect.element(darkOption).toHaveAttribute('aria-pressed', 'false');

		await darkOption.click();
		expect(ontheme).toHaveBeenCalledWith('dark');
		await expect.element(dialog).not.toBeInTheDocument();
	});

	it('reflects light theme with pressed light option', async () => {
		const ontheme = vi.fn();
		await render(ThemePickerButton, { theme: 'light', ontheme });

		await page.getByRole('button', { name: 'Theme' }).click();

		const lightOption = page.getByRole('button', { name: 'Light' });
		await expect.element(lightOption).toHaveAttribute('aria-pressed', 'true');

		const systemOption = page.getByRole('button', { name: 'System' });
		await systemOption.click();
		expect(ontheme).toHaveBeenCalledWith('system');
	});

	it('reflects dark theme with pressed dark option', async () => {
		const ontheme = vi.fn();
		await render(ThemePickerButton, { theme: 'dark', ontheme });

		await page.getByRole('button', { name: 'Theme' }).click();

		const darkOption = page.getByRole('button', { name: 'Dark' });
		await expect.element(darkOption).toHaveAttribute('aria-pressed', 'true');

		const lightOption = page.getByRole('button', { name: 'Light' });
		await lightOption.click();
		expect(ontheme).toHaveBeenCalledWith('light');
	});

	it('consumes theme and changeTheme from context', async () => {
		const changeTheme = vi.fn();
		await render(ContextFixture, { theme: 'dark', changeTheme });

		await page.getByRole('button', { name: 'Theme' }).click();

		const darkOption = page.getByRole('button', { name: 'Dark' });
		await expect.element(darkOption).toHaveAttribute('aria-pressed', 'true');

		await page.getByRole('button', { name: 'Light' }).click();
		expect(changeTheme).toHaveBeenCalledWith('light');
	});
});
