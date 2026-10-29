import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
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

	it('steps back in a card footer: quiet, regular weight, still with its arrow', async () => {
		await render(Link, { href: 'https://docs.google.com', label: 'Open in Sheets', quiet: true });
		const link = page.getByRole('link', { name: 'Open in Sheets' }).element();
		const style = getComputedStyle(link);
		expect(style.fontWeight).toBe('400');
		expect(style.color).toBe(tokenColor('nm-text-2'));
		expect(link.querySelector('svg.lucide-arrow-up-right')).not.toBeNull();
	});

	it('sits inside a sentence in text-2, as in the pen, and underlines on hover', async () => {
		await render(Link, { href: '/settings/credits', label: 'switch to Nomi credits', variant: 'inline' });
		const link = page.getByRole('link', { name: 'switch to Nomi credits' });
		const style = getComputedStyle(link.element());
		expect(style.color).toBe(tokenColor('nm-text-2'));
		expect(style.textDecorationLine).toBe('none');
		expect(link.element().querySelector('svg')).toBeNull();
		await userEvent.hover(link);
		expect(getComputedStyle(link.element()).textDecorationLine).toBe('underline');
	});

	it('opens other sites in a new tab without handing them this page', async () => {
		await render(Link, { href: 'https://platform.deepseek.com', label: 'Add credits on DeepSeek', external: true });
		const link = page.getByRole('link', { name: /Add credits on DeepSeek/ }).element();
		expect(link.getAttribute('target')).toBe('_blank');
		expect(link.getAttribute('rel')).toBe('noopener noreferrer');
	});
});
