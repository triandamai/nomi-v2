import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ThinkingSection from './ThinkingSection.svelte';

describe('Thinking & planning section', () => {
	it('shows reasoning, thoughts, a running plan and a draft that can be edited', async () => {
		await render(ThinkingSection);
		await expect.element(page.getByText('Weekly money check')).toBeInTheDocument();
		await expect.element(page.getByText('Worked through 4 steps')).toBeInTheDocument();
		await page.getByRole('button', { name: 'Remove step 5' }).click();
		await expect.element(page.getByText("File each month's receipt")).not.toBeInTheDocument();
	});
});
