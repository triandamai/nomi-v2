import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import TogetherSection from './TogetherSection.svelte';

describe('Build together section', () => {
	it('shows people in the chat, asks in line, roles, the invite and live sessions', async () => {
		await render(TogetherSection);
		await expect.element(page.getByText('Sari is typing')).toBeInTheDocument();
		await expect.element(page.getByText('2 asks in line')).toBeInTheDocument();
		await expect.element(page.getByRole('radiogroup', { name: 'Role' })).toBeInTheDocument();
		await expect.element(page.getByRole('link', { name: 'Join and build' })).toBeInTheDocument();
		await expect.element(page.getByText('Following Sari · Code · +page.svelte')).toBeInTheDocument();
	});
});
