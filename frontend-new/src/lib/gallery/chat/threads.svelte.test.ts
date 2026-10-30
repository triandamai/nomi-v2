import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ThreadsSection from './ThreadsSection.svelte';

describe('Chat management & long threads section', () => {
	it('shows the chat menu and move dialog open, rename, and the long-thread markers', async () => {
		await render(ThreadsSection);
		// The chat menu and the move dialog are drawn open, as on the sheet.
		await expect.element(page.getByRole('menuitem', { name: 'Delete chat' })).toBeVisible();
		await expect.element(page.getByRole('heading', { name: 'Move chat to a project' })).toBeVisible();
		await expect.element(page.getByRole('radio', { name: /budget-keluarga/ })).toBeChecked();
		await expect.element(page.getByRole('textbox', { name: 'Chat name' })).toHaveValue('Food spending in October');
		await expect.element(page.getByRole('button', { name: '2 new replies' })).toBeInTheDocument();
		await expect.element(page.getByRole('separator', { name: 'Yesterday' })).toBeInTheDocument();
	});
});
