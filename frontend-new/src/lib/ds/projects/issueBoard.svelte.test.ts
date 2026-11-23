import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import IssueBoard from './IssueBoard.svelte';
import { tokenColor } from '../test-utils';

// The workspace's Issues sheet: columns of cards that move by drag or by their Move to select.

const labels = { board: 'Issue board', empty: 'No issues here', moveTo: (number: number) => `Move #${number} to`, retry: 'Try again', openChat: 'Open its chat' };
const columns = [
	{ id: 'backlog', name: 'Backlog', cards: [{ number: 1, title: 'Fix the slot picker', tags: 'Bug · High', status: { text: 'Overdue since 18 Nov', tone: 'late' as const } }] },
	{ id: 'ready', name: 'Ready to dev', hint: 'The crew picks these up in order.', cards: [{ number: 2, title: 'QRIS upload', tags: 'Task · Medium', status: { text: 'Koda couldn’t finish it', tone: 'failed' as const, crew: 'koda' as const }, retry: true, chatHref: '/chat/s2' }] },
	{ id: 'done', name: 'Done', cards: [] },
];

describe('IssueBoard', () => {
	it('shows each column with its cards, and says when one is empty', async () => {
		await render(IssueBoard, { columns, labels, onmove: () => {} });
		await expect.element(page.getByRole('region', { name: 'Backlog' })).toBeVisible();
		await expect.element(page.getByText('Fix the slot picker')).toBeVisible();
		await expect.element(page.getByText('No issues here')).toBeVisible();
		expect(getComputedStyle(page.getByText('Overdue since 18 Nov').element()).color).toBe(tokenColor('nm-warning'));
		expect(getComputedStyle(page.getByText('Koda couldn’t finish it').element()).color).toBe(tokenColor('nm-danger'));
		await expect.element(page.getByRole('link', { name: 'Open its chat' })).toHaveAttribute('href', '/chat/s2');
	});

	it('moves a card with its select, to the end of the column', async () => {
		const onmove = vi.fn();
		await render(IssueBoard, { columns, labels, onmove });
		await page.getByRole('combobox', { name: 'Move #1 to' }).selectOptions('Ready to dev');
		expect(onmove).toHaveBeenCalledWith(1, 'ready', 1);
	});

	it('hands a stuck card back to the crew', async () => {
		const onretry = vi.fn();
		await render(IssueBoard, { columns, labels, onmove: () => {}, onretry });
		await page.getByRole('button', { name: 'Try again' }).click();
		expect(onretry).toHaveBeenCalledWith(2);
	});

	it('moves a card dropped on another column', async () => {
		const onmove = vi.fn();
		await render(IssueBoard, { columns, labels, onmove });
		const card = page.getByText('Fix the slot picker').element().closest('[data-card]') as HTMLElement;
		const target = page.getByRole('region', { name: 'Done' }).element() as HTMLElement;
		const data = new DataTransfer();
		card.dispatchEvent(new DragEvent('dragstart', { bubbles: true, dataTransfer: data }));
		target.dispatchEvent(new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: data }));
		target.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: data, clientY: 10 }));
		expect(onmove).toHaveBeenCalledWith(1, 'done', 0);
	});
});
