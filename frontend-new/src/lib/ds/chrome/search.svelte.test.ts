import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import QuickSearch from './QuickSearch.svelte';
import { tokenColor } from '../test-utils';

afterEach(() => {
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const labels = { field: 'Search chats, memory, files', groups: { chat: 'Chats', memory: 'Memory', file: 'Files' }, empty: 'Nothing found for “zz”', searching: 'Searching…', failed: 'Search isn’t answering. Try again in a moment.' };
const groups = [
	{ kind: 'chat' as const, rows: [{ id: 'c1', title: 'Budget belanja Oktober', snippet: '…beras naik 10%…', meta: '2h ago', href: '/chat/s1' }] },
	{ kind: 'file' as const, rows: [{ id: 'f1', title: 'src/routes/menu/+page.svelte', meta: '1d ago', href: '/projects/p1?open=code&file=x' }] },
];

describe('QuickSearch', () => {
	it('is the pen’s 240 × 30 field in the top bar, with its ⌘K', async () => {
		await page.viewport(1512, 862);
		await render(QuickSearch, { labels, shortcut: '⌘K', groups: null });
		const field = page.getByRole('combobox', { name: 'Search chats, memory, files' }).element();
		const frame = field.closest('[data-part="field"]')!;
		expect([box(frame).width, box(frame).height, getComputedStyle(frame).borderColor]).toEqual([240, 30, tokenColor('nm-border')]);
		await expect.element(page.getByText('⌘K')).toBeVisible();
		expect(field.getAttribute('aria-expanded')).toBe('false');
	});

	it('shows what was found in its groups, and opens one with the keys or a click', async () => {
		const onpick = vi.fn();
		const oninput = vi.fn();
		await render(QuickSearch, { labels, groups, value: 'belanja', onpick, oninput });
		const field = page.getByRole('combobox', { name: 'Search chats, memory, files' });
		await field.click();
		await expect.element(page.getByRole('listbox')).toBeVisible();
		await expect.element(page.getByRole('group', { name: 'Chats' }).getByRole('option', { name: /Budget belanja Oktober/ })).toBeVisible();
		await expect.element(page.getByRole('group', { name: 'Files' })).toBeVisible();
		const panel = page.getByRole('listbox').element();
		expect(getComputedStyle(panel).boxShadow).not.toBe('none');
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(field).toHaveAttribute('aria-activedescendant', page.getByRole('option', { name: /Budget/ }).element().id);
		await userEvent.keyboard('{ArrowDown}{Enter}');
		expect(onpick).toHaveBeenCalledWith(groups[1].rows[0]);
		await field.fill('kopi');
		expect(oninput).toHaveBeenLastCalledWith('kopi');
	});

	it('closes on Escape, and says when nothing was found, while it looks, or that it couldn’t', async () => {
		const screen = await render(QuickSearch, { labels, groups: [], value: 'zz' });
		await page.getByRole('combobox').click();
		await expect.element(page.getByText('Nothing found for “zz”')).toBeVisible();
		await userEvent.keyboard('{Escape}');
		expect(page.getByText('Nothing found for “zz”').elements()).toHaveLength(0);
		await screen.rerender({ labels, groups: null, value: 'zz', searching: true });
		await page.getByRole('combobox').click();
		await expect.element(page.getByText('Searching…')).toBeVisible();
		await screen.rerender({ labels, groups: null, value: 'zz', failed: true });
		await expect.element(page.getByText('Search isn’t answering. Try again in a moment.')).toBeVisible();
	});
});
