import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import MobileSection from './MobileSection.svelte';

describe('Mobile chat section', () => {
	it('shows the shell, messages, buttons and cards, sheets, decisions and the chats list', async () => {
		await render(MobileSection);
		await expect.element(page.getByRole('heading', { name: 'Food spending in October' })).toBeInTheDocument();
		await expect.element(page.getByText('Food by week')).toBeInTheDocument();
		await expect.element(page.getByText('DeepSeek said')).toBeInTheDocument();
		await expect.element(page.getByRole('heading', { name: 'Reply' })).toBeInTheDocument();
		await expect.element(page.getByRole('progressbar')).toBeInTheDocument();
		await page.getByRole('radio', { name: /Whoosh high-speed train/ }).click();
		await expect.element(page.getByRole('radio', { name: /Whoosh high-speed train/ })).toBeChecked();
		await expect.element(page.getByRole('link', { name: /Food spending in October/ })).toBeInTheDocument();
	});
});
