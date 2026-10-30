import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import BuildingSection from './BuildingSection.svelte';

describe('Building section', () => {
	it('shows the file list and the preview while building', async () => {
		await render(BuildingSection);
		await expect.element(page.getByText('5 of 9 written')).toBeInTheDocument();
		await expect.element(page.getByRole('heading', { name: 'Building your app' })).toBeInTheDocument();
	});
});
