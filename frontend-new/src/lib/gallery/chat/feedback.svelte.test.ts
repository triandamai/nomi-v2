import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import FeedbackSection from './FeedbackSection.svelte';

describe('Response feedback section', () => {
	it('shows the reply bar states, dislike, share and delete, working', async () => {
		await render(FeedbackSection);
		expect(page.getByRole('group', { name: 'Reply actions' }).elements().length).toBeGreaterThanOrEqual(3);
		await expect.element(page.getByText('What went wrong?')).toBeInTheDocument();
		await expect.element(page.getByText(/Rp ••• against Rp •••/)).toBeInTheDocument();
		await page.getByRole('switch', { name: 'Hide amounts and names' }).click();
		await expect.element(page.getByText(/Rp 2,1 jt against Rp 1,4 jt/)).toBeInTheDocument();
		await expect.element(page.getByText('Link copied')).toBeInTheDocument();
		await page.getByRole('button', { name: 'Delete this reply…' }).click();
		await expect.element(page.getByRole('dialog', { name: 'Delete this reply?' })).toBeVisible();
	});
});
