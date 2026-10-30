import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import RichSection from './RichSection.svelte';

describe('Rich content section', () => {
	it('shows the table, cart, code, diff, links and mentions boards, working', async () => {
		await render(RichSection);
		await expect.element(page.getByRole('table', { name: 'Food spending by week' })).toBeInTheDocument();
		await expect.element(page.getByText('Koda wants to change 1 file')).toBeInTheDocument();
		await expect.element(page.getByRole('link', { name: /Kos Mawar Tebet — kamar AC, kamar mandi dalam/ })).toBeInTheDocument();
		await expect.element(page.getByRole('group', { name: 'Quantity of Telur ayam negeri' })).toHaveTextContent('2');
		await page.getByRole('button', { name: 'More Telur ayam negeri' }).click();
		await expect.element(page.getByRole('group', { name: 'Quantity of Telur ayam negeri' })).toHaveTextContent('3');
		await userEvent.hover(page.getByRole('option', { name: /Maya/ }));
		await expect.element(page.getByRole('option', { name: /Maya/ })).toHaveAttribute('aria-selected', 'true');
		await expect.element(page.getByText('due now · 25 Nov 09:00 · every month')).toBeInTheDocument();
	});
});
