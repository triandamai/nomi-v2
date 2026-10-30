import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import CodeSection from './CodeSection.svelte';

describe('Code, terminal & phone section', () => {
	it('shows the code view, both terminal runs, the phone preview and the code quote', async () => {
		await page.viewport(1280, 900);
		await render(CodeSection);
		expect(page.getByRole('tree').elements().length).toBe(2);
		await expect.element(page.getByRole('button', { name: 'Ask Koda about lines 14–15' })).toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: 'Ask Koda to fix' })).toBeInTheDocument();
		await expect.element(page.getByText('Warung Kopi Bu Siti', { exact: true }).first()).toBeInTheDocument();
		await expect.element(page.getByText('+page.svelte · lines 14–15')).toBeInTheDocument();
	});
});
