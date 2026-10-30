import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import WorkspaceSection from './WorkspaceSection.svelte';

describe('Workspace section', () => {
	it('shows the resize handle states, the four publish states and the sheet', async () => {
		await render(WorkspaceSection);
		expect(page.getByRole('separator', { name: 'Resize chat' }).elements()).toHaveLength(3);
		await expect.element(page.getByRole('button', { name: 'Up to date' })).toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: 'Publish failed · Retry' })).toBeInTheDocument();
		await expect.element(page.getByRole('region', { name: 'Domain' })).toBeInTheDocument();
	});
});
