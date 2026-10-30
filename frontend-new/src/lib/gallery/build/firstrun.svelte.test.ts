import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import FirstRunSection from './FirstRunSection.svelte';

describe('First run section', () => {
	it('shows the first-run card', async () => {
		await render(FirstRunSection);
		await expect.element(page.getByRole('heading', { name: 'What should we call you?' })).toBeInTheDocument();
	});
});
