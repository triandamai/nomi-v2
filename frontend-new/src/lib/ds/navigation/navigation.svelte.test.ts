import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { Folder, MessageSquare } from '@lucide/svelte';
import NavItem from './NavItem.svelte';
import NavGroupLabel from './NavGroupLabel.svelte';
import RecentRow from './RecentRow.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Sidebar parts. Active = surface-2 fill + primary text.

describe('NavItem', () => {
	it('is a 32px link with its icon, label and count', async () => {
		await render(NavItem, { href: '/chats', icon: MessageSquare, label: 'Chats', count: 12 });
		const link = page.getByRole('link', { name: /Chats/ });
		await expect.element(link).toHaveAttribute('href', '/chats');
		const style = getComputedStyle(link.element());
		expect(style.height).toBe('32px');
		expect(style.color).toBe(tokenColor('nm-text-2'));
		expect(style.backgroundColor).toBe('rgba(0, 0, 0, 0)');
		expect(getComputedStyle(page.getByText('12').element()).fontFamily).toContain('Geist Mono');
	});

	it('marks where the person is', async () => {
		await render(NavItem, { href: '/projects', icon: Folder, label: 'Projects', count: 4, active: true });
		const link = page.getByRole('link', { name: /Projects/ });
		await expect.element(link).toHaveAttribute('aria-current', 'page');
		const style = getComputedStyle(link.element());
		expect(style.backgroundColor).toBe(tokenBackground('nm-surface-2'));
		expect(style.color).toBe(tokenColor('nm-text'));
		expect(style.fontWeight).toBe('500');
	});
});

describe('NavGroupLabel', () => {
	it('names a group in the mono label style', async () => {
		await render(NavGroupLabel, { label: 'Build' });
		const label = page.getByText('Build').element();
		const style = getComputedStyle(label);
		expect(style.textTransform).toBe('uppercase');
		expect(style.fontFamily).toContain('Geist Mono');
		expect(style.color).toBe(tokenColor('nm-text-3'));
	});
});

describe('RecentRow', () => {
	it('links to a recent chat or project with how long ago, in mono', async () => {
		await render(RecentRow, { href: '/chat/1', kind: 'chat', title: 'Budget belanja Oktober', when: '2h' });
		const link = page.getByRole('link', { name: /Budget belanja Oktober/ });
		expect(getComputedStyle(link.element()).height).toBe('30px');
		expect(link.element().querySelector('svg.lucide-message-square')).not.toBeNull();
		expect(getComputedStyle(page.getByText('2h').element()).fontFamily).toContain('Geist Mono');

		await render(RecentRow, { href: '/projects/2', kind: 'project', title: 'salah-daily-tracker', when: '2d' });
		expect(page.getByRole('link', { name: /salah-daily-tracker/ }).element().querySelector('svg.lucide-folder')).not.toBeNull();
	});

	it('keeps a long title to one line', async () => {
		await render(RecentRow, { href: '/chat/1', kind: 'chat', title: 'A very long chat title that goes on and on past the sidebar edge', when: '3d' });
		expect(getComputedStyle(page.getByText(/A very long chat title/).element()).textOverflow).toBe('ellipsis');
	});
});
