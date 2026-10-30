import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Checkbox from './Checkbox.svelte';
import { tokenBackground } from '../test-utils';

describe('Checkbox', () => {
	it('is a real checkbox with its label, filled in text colour when checked', async () => {
		await render(Checkbox, { label: 'Mask amounts and names', checked: true });
		const box = page.getByRole('checkbox', { name: 'Mask amounts and names' });
		await expect.element(box).toBeChecked();
		const style = getComputedStyle(box.element());
		expect([style.width, style.height]).toEqual(['16px', '16px']);
		expect(style.backgroundColor).toBe(tokenBackground('nm-text'));
		expect(style.borderTopLeftRadius).toBe('4px');
	});

	it('toggles from its label', async () => {
		let checked = false;
		await render(Checkbox, { label: 'Mask amounts and names', onchange: (value: boolean) => (checked = value) });
		await page.getByText('Mask amounts and names').click();
		expect(checked).toBe(true);
		await expect.element(page.getByRole('checkbox')).toBeChecked();
	});
});
