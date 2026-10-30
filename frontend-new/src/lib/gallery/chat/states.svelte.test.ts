import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import StatesSection from './StatesSection.svelte';

describe('States & errors section', () => {
	it('shows message states, banners and every provider error with its next move', async () => {
		await render(StatesSection);
		await expect.element(page.getByText('not sent')).toBeInTheDocument();
		await expect.element(page.getByText('Back online. 1 queued message sent.')).toBeInTheDocument();
		for (const title of ['Key rejected', 'Provider error', 'Model not found', 'Rate limited', 'Provider unavailable', 'Timed out', 'Context limit', 'Blocked by provider', 'Attachment rejected', "Can't connect", 'Tool failed · gmail.send', 'Out of Nomi credits']) {
			await expect.element(page.getByText(title, { exact: true })).toBeInTheDocument();
		}
		expect(page.getByRole('alert').elements()).toHaveLength(12);
	});
});
