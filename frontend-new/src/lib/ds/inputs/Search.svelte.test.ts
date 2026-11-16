import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import Search from './Search.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

describe('Search', () => {
	it('is a 30px search box with its shortcut shown', async () => {
		await render(Search, { placeholder: 'Search chats, memory, files', shortcut: '⌘K' });
		const box = page.getByRole('searchbox', { name: 'Search chats, memory, files' });
		await expect.element(box).toBeInTheDocument();
		const frame = getComputedStyle(box.element().closest('label')!);
		expect(frame.height).toBe('30px');
		expect(frame.borderTopColor).toBe(tokenColor('nm-border'));
		await expect.element(page.getByText('⌘K')).toBeInTheDocument();
	});

	it('reports what is typed', async () => {
		let query = '';
		await render(Search, { placeholder: 'Search', oninput: (value: string) => (query = value) });
		await userEvent.type(page.getByRole('searchbox'), 'kopi');
		expect(query).toBe('kopi');
	});
});

describe('Search on a phone', () => {
	it('is the pen’s 38 pill on surface-2 with 16pt text, so iOS doesn’t zoom', async () => {
		await render(Search, { placeholder: 'Search chats', phone: true });
		const field = page.getByRole('searchbox', { name: 'Search chats' }).element();
		const frame = field.closest('label')!;
		const style = getComputedStyle(frame);
		expect([frame.getBoundingClientRect().height, style.borderRadius, style.backgroundColor, getComputedStyle(field).fontSize]).toEqual([38, '12px', tokenBackground('nm-surface-2'), '16px']);
	});
});
