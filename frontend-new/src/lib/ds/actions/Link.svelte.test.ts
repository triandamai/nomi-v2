import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Link from './Link.svelte';
import { tokenColor } from '../test-utils';

describe('Link', () => {
	it('stands on its own in 13px / 500 with an arrow', async () => {
		await render(Link, { href: '/settings/plan', label: 'Upgrade to Pro' });
		const link = page.getByRole('link', { name: 'Upgrade to Pro' }).element();
		const style = getComputedStyle(link);
		expect(style.fontWeight).toBe('500');
		expect(style.color).toBe(tokenColor('nm-text'));
		expect(link.querySelector('svg.lucide-arrow-up-right')?.getAttribute('width')).toBe('14');
	});

	it('sits inside a sentence underlined, so it is not told apart by colour alone', async () => {
		await render(Link, { href: '/settings/credits', label: 'switch to Nomi credits', variant: 'inline' });
		const link = page.getByRole('link', { name: 'switch to Nomi credits' }).element();
		const style = getComputedStyle(link);
		expect(style.color).toBe(tokenColor('nm-text-2'));
		expect(style.textDecorationLine).toBe('underline');
		expect(link.querySelector('svg')).toBeNull();
	});

	it('opens other sites in a new tab without handing them this page', async () => {
		await render(Link, { href: 'https://platform.deepseek.com', label: 'Add credits on DeepSeek', external: true });
		const link = page.getByRole('link', { name: /Add credits on DeepSeek/ }).element();
		expect(link.getAttribute('target')).toBe('_blank');
		expect(link.getAttribute('rel')).toBe('noopener noreferrer');
	});
});
