import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import TooltipFixture from './TooltipFixture.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

describe('Tooltip', () => {
	it('appears on hover with the shortcut, inverted', async () => {
		await render(TooltipFixture);
		await expect.element(page.getByRole('tooltip')).not.toBeInTheDocument();
		await userEvent.hover(page.getByRole('button', { name: 'New chat' }));
		const tip = page.getByRole('tooltip');
		await expect.element(tip).toBeVisible();
		await expect.element(tip).toHaveTextContent(/^New chat\s*⌘K$/);
		expect(getComputedStyle(tip.element()).backgroundColor).toBe(tokenBackground('nm-text'));
		expect(getComputedStyle(page.getByText('⌘K').element()).color).toBe(tokenColor('nm-text-3'));
		await userEvent.unhover(page.getByRole('button', { name: 'New chat' }));
		await expect.element(page.getByRole('tooltip')).not.toBeInTheDocument();
	});

	it('appears on keyboard focus, describes its button, and Escape hides it', async () => {
		await render(TooltipFixture);
		await userEvent.tab();
		const tip = page.getByRole('tooltip');
		await expect.element(tip).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'New chat' })).toHaveAttribute('aria-describedby', tip.element().id);
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('tooltip')).not.toBeInTheDocument();
	});
});
