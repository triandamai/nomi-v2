import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
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

	it('edits a cell in place: Enter saves, Esc puts it back, and the key’s cells stay as they are', async () => {
		await page.viewport(1512, 862);
		const onsave = vi.fn(async () => true);
		await render(RowsGrid, {
			label: 'orders',
			columns: ['id', 'status'],
			rows: [['0007', 'new']],
			edit: { locked: [0], initial: () => 'new', onsave, cellLabel: (row: number, column: number) => `Edit ${['id', 'status'][column]} of row ${row + 1}` },
		});
		expect(page.getByRole('button', { name: 'Edit id of row 1' }).elements()).toHaveLength(0);
		const cell = page.getByRole('button', { name: 'Edit status of row 1' });
		await cell.click();
		const field = page.getByRole('textbox', { name: 'Edit status of row 1' });
		await expect.element(field).toHaveValue('new');
		const row = page.getByRole('row', { name: /0007/ }).element();
		expect(box(row).height).toBe(32);
		await field.fill('picked up');
		await userEvent.keyboard('{Escape}');
		expect(onsave).not.toHaveBeenCalled();
		await expect.element(page.getByRole('button', { name: 'Edit status of row 1' })).toBeVisible();
		await page.getByRole('button', { name: 'Edit status of row 1' }).click();
		await page.getByRole('textbox', { name: 'Edit status of row 1' }).fill('picked up');
		await userEvent.keyboard('{Enter}');
		expect(onsave).toHaveBeenCalledWith(0, 1, 'picked up');
		await expect.element(page.getByRole('button', { name: 'Edit status of row 1' })).toBeVisible();
	});

	it('keeps the field open when the save is refused', async () => {
		await render(RowsGrid, { label: 'orders', columns: ['id', 'total'], rows: [['1', '10']], edit: { locked: [0], initial: () => '10', onsave: async () => false, cellLabel: () => 'Edit total' } });
		await page.getByRole('button', { name: 'Edit total' }).click();
		await page.getByRole('textbox', { name: 'Edit total' }).fill('abc');
		await userEvent.keyboard('{Enter}');
		await expect.element(page.getByRole('textbox', { name: 'Edit total' })).toHaveValue('abc');
	});
});
