import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import FirstRun from './FirstRun.svelte';

// Build → First run: one screen after the code: a name and one line on what Nomi does. No crew
// tour; the composer comes next.

describe('FirstRun', () => {
	it('asks one thing, a name, says what Nomi does, and continues', async () => {
		const oncontinue = vi.fn();
		await render(FirstRun, { name: 'Mario Prasetyo', oncontinue });
		await expect.element(page.getByRole('heading', { name: 'What should we call you?' })).toBeVisible();
		await expect.element(page.getByText('Nomi builds real web apps from a description.', { exact: false })).toBeVisible();
		const field = page.getByRole('textbox', { name: 'Your name' });
		await expect.element(field).toHaveValue('Mario Prasetyo');
		expect(getComputedStyle(page.getByText('Shown on your projects. Change it in Settings › Profile.').element()).fontSize).toBe('11px');
		await field.fill('Mario');
		await page.getByRole('button', { name: 'Continue' }).click();
		expect(oncontinue).toHaveBeenCalledWith('Mario');
	});

	it('does not continue without a name', async () => {
		const oncontinue = vi.fn();
		await render(FirstRun, { oncontinue });
		await expect.element(page.getByRole('button', { name: 'Continue' })).toBeDisabled();
	});
});
