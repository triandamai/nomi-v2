import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ThreadsSection from './ThreadsSection.svelte';

describe('Chat management & long threads section', () => {
	it('shows the chat menu, rename, move and the long-thread markers, working', async () => {
		await render(ThreadsSection);
		await page.getByRole('button', { name: 'Chat options' }).click();
		await expect.element(page.getByRole('menuitem', { name: 'Delete chat' })).toBeVisible();
		await page.getByRole('menuitem', { name: 'Move to a project' }).click();
		await expect.element(page.getByRole('dialog', { name: 'Move chat to a project' })).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		await expect.element(page.getByRole('textbox', { name: 'Chat name' })).toHaveValue('Food spending in October');
		await expect.element(page.getByRole('button', { name: '2 new replies' })).toBeInTheDocument();
		await expect.element(page.getByRole('separator', { name: 'Yesterday' })).toBeInTheDocument();
	});
});
