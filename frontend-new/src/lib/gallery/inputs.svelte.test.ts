import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import InputsSection from './InputsSection.svelte';
import AgentsSection from './AgentsSection.svelte';

describe('Inputs & selection section', () => {
	it('lays out every input and state from the design sheet', async () => {
		await render(InputsSection);
		for (const caption of [
			'Field / Default',
			'Field / Focus',
			'Field / Error',
			'Field / Disabled',
			'Search',
			'Chip / Model',
			'Chip / Option',
			'Chip / Agent',
			'Chip / Provision',
			'Segmented (composed)',
			'Toggle / On · Off',
			'Check / Checked · Unchecked',
		]) {
			expect(page.getByText(caption, { exact: true }).elements(), caption).toHaveLength(1);
		}
		await expect.element(page.getByRole('textbox', { name: 'Message' })).toBeInTheDocument();
	});
});

describe('Agents section', () => {
	it('shows each crew member at 96, 32 and 20px', async () => {
		const { container } = await render(AgentsSection);
		expect(container.querySelectorAll('svg[width="96"]')).toHaveLength(10);
		expect(container.querySelectorAll('svg[width="32"]')).toHaveLength(10);
		expect(container.querySelectorAll('svg[width="20"]')).toHaveLength(10);
	});
});
