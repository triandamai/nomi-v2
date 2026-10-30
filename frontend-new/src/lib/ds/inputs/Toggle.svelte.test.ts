import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import Toggle from './Toggle.svelte';
import { tokenBackground } from '../test-utils';

describe('Toggle', () => {
	it('is a switch: mint and knob right when on, quiet when off', async () => {
		await render(Toggle, { label: 'Weekly summary', checked: true });
		const on = page.getByRole('switch', { name: 'Weekly summary' });
		await expect.element(on).toHaveAttribute('aria-checked', 'true');
		const style = getComputedStyle(on.element());
		expect([style.width, style.height]).toEqual(['32px', '18px']);
		expect(style.backgroundColor).toBe(tokenBackground('nm-accent'));
	});

	it('flips by click or Space', async () => {
		let checked = false;
		await render(Toggle, { label: 'Weekly summary', checked: false, onchange: (value: boolean) => (checked = value) });
		const toggle = page.getByRole('switch', { name: 'Weekly summary' });
		expect(getComputedStyle(toggle.element()).backgroundColor).toBe(tokenBackground('nm-surface-2'));
		await toggle.click();
		expect(checked).toBe(true);
		await expect.element(toggle).toHaveAttribute('aria-checked', 'true');
		await userEvent.keyboard(' ');
		expect(checked).toBe(false);
	});
});
