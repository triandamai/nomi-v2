import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ShellsSection from './ShellsSection.svelte';
import RulesSection from './RulesSection.svelte';

describe('Layout shells section', () => {
	it('draws the three shells with their fixed columns', async () => {
		await render(ShellsSection);
		for (const text of ['App shell', 'Workspace shell', 'Focus shell', 'Sidebar 248', 'Agent chat 440', 'Column 480–720']) {
			expect(page.getByText(text, { exact: true }).elements(), text).toHaveLength(1);
		}
	});
});

describe('Rules section', () => {
	it('lists what to do and what not to, from the design sheet', async () => {
		await render(RulesSection);
		await expect.element(page.getByText('Use nm-* tokens for every colour, size, gap and radius')).toBeInTheDocument();
		await expect.element(page.getByText('Pure #000 or #FFF surfaces')).toBeInTheDocument();
	});
});
