import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import TableChip from './TableChip.svelte';
import RowsGrid from './RowsGrid.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Database (Flow / Build — Workspace tools U16wU, Screen / Database xYgNi), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('TableChip', () => {
	it('is a 29px chip: the table’s name in mono and its rows, the chosen one filled', async () => {
		const onclick = vi.fn();
		await render(TableChip, { name: 'orders', rows: '7 rows', selected: true, onclick });
		const chip = page.getByRole('button', { name: /orders/ }).element();
		expect([box(chip).height, getComputedStyle(chip).backgroundColor, chip.getAttribute('aria-pressed')]).toEqual([29, tokenBackground('nm-surface-2'), 'true']);
		await page.getByRole('button', { name: /orders/ }).click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});

describe('RowsGrid', () => {
	it('is the pen’s outlined grid: the head on surface-2, 32px mono rows over their lines', async () => {
		await page.viewport(1512, 862);
		await render(RowsGrid, { label: 'orders', columns: ['id', 'name'], rows: [['0007', 'Rina'], ['0006', 'Pak Hadi']] });
		const table = page.getByRole('table', { name: 'orders' }).element();
		expect(getComputedStyle(table.parentElement!).outlineColor).toBe(tokenColor('nm-border'));
		const head = table.querySelector('thead tr')!;
		expect([box(head).height, getComputedStyle(head).backgroundColor]).toEqual([32, tokenBackground('nm-surface-2')]);
		const row = page.getByRole('row', { name: /Rina/ }).element();
		expect([box(row).height, getComputedStyle(row).borderTopColor]).toEqual([32, tokenColor('nm-border')]);
		expect(getComputedStyle(page.getByText('Pak Hadi').element()).fontFamily).toMatch(/Mono/);
		await expect.element(page.getByRole('columnheader', { name: 'name' })).toBeVisible();
	});
});
