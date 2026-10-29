import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ActionsSection from './ActionsSection.svelte';

describe('Actions section', () => {
	it('shows approval, decision, selection, form, suggestions and clarify boards, working', async () => {
		await render(ActionsSection);
		await expect.element(page.getByText('Tara wants to send an email')).toBeInTheDocument();
		await expect.element(page.getByText('3 selected · Rp 285.990 / month')).toBeInTheDocument();
		await page.getByRole('checkbox', { name: /Fitness First/ }).click();
		await expect.element(page.getByText('4 selected · Rp 935.990 / month')).toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: 'Track 4' })).toBeInTheDocument();
		await expect.element(page.getByText('Ready to run this plan?')).toBeInTheDocument();
		await expect.element(page.getByText('Usage notes')).toBeInTheDocument();
	});
});
