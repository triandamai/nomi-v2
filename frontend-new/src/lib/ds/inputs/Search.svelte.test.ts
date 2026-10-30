import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import Search from './Search.svelte';
import { tokenColor } from '../test-utils';

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
