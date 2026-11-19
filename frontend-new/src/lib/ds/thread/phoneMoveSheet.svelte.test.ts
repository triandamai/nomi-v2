import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import PhoneMoveSheet from './PhoneMoveSheet.svelte';

const projects = [
	{ id: 'p1', name: 'budget-keluarga', workspace: 'Keluarga workspace', shared: 'Keluarga' },
	{ id: 'p2', name: 'salah-daily-tracker', workspace: 'Personal workspace' },
];

// Flow / Mobile Chat — Manage chat, step 3 (x73ZO): a 40 search, 54 rows (the name in mono at 14),
// the chosen one filled with its check, the note on who gains access, and Move chat at 46.
describe('PhoneMoveSheet', () => {
	it('lists the projects at the pen’s sizes and says who will see a shared one', async () => {
		await render(PhoneMoveSheet, { open: true, projects, onmove: () => {} });
		const row = page.getByRole('radio', { name: /budget-keluarga/ });
		expect(row.element().getBoundingClientRect().height).toBe(54);
		expect(getComputedStyle(page.getByText('budget-keluarga').element()).fontFamily).toMatch(/mono/i);
		expect(page.getByRole('searchbox', { name: 'Search projects' }).element().parentElement?.getBoundingClientRect().height).toBe(40);
		await expect.element(page.getByRole('button', { name: 'Move chat' })).toBeDisabled();
		await row.click();
		await expect.element(row).toHaveAttribute('aria-checked', 'true');
		await expect.element(page.getByText('Keluarga members will see this chat. Memory and money records stay personal.')).toBeVisible();
		expect(page.getByRole('button', { name: 'Move chat' }).element().getBoundingClientRect().height).toBe(46);
	});

	it('filters as it’s typed and moves to the one picked', async () => {
		const onmove = vi.fn();
		await render(PhoneMoveSheet, { open: true, projects, current: 'p1', onmove });
		await userEvent.type(page.getByRole('searchbox', { name: 'Search projects' }), 'salah');
		await expect.element(page.getByRole('radio', { name: /budget-keluarga/ })).not.toBeInTheDocument();
		await page.getByRole('radio', { name: /salah-daily-tracker/ }).click();
		await page.getByRole('button', { name: 'Move chat' }).click();
		expect(onmove).toHaveBeenCalledWith('p2');
	});
});
