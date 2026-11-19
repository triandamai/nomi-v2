import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import PhoneDeviceToggle from './PhoneDeviceToggle.svelte';

// Flow / Mobile Build → step 5 (PsC5F) → Device Toggle G0ih1: 40 high, inset 2, radius 10, each
// choice 12 to the side with its 14 icon and its word at 13; the chosen one on bg at 500.
describe('PhoneDeviceToggle', () => {
	it('is the pen’s 40-high pill with an icon and word on each side', async () => {
		render(PhoneDeviceToggle, { value: 'desktop' });
		const group = page.getByRole('radiogroup', { name: 'Device' });
		const box = group.element().getBoundingClientRect();
		expect(box.height).toBe(40);
		expect(getComputedStyle(group.element()).borderRadius).toBe('10px');
		const desktop = page.getByRole('radio', { name: 'Desktop' });
		await expect.element(desktop).toHaveAttribute('aria-checked', 'true');
		expect(getComputedStyle(desktop.element()).fontWeight).toBe('500');
		expect(getComputedStyle(desktop.element()).paddingLeft).toBe('12px');
		expect(desktop.element().querySelector('svg')?.getAttribute('width')).toBe('14');
		await expect.element(page.getByRole('radio', { name: 'Phone' })).toHaveAttribute('aria-checked', 'false');
	});

	it('changes on a tap and with the arrow keys', async () => {
		const onchange = vi.fn();
		render(PhoneDeviceToggle, { value: 'phone', onchange });
		await page.getByRole('radio', { name: 'Desktop' }).click();
		expect(onchange).toHaveBeenLastCalledWith('desktop');
		await userEvent.keyboard('{ArrowLeft}');
		expect(onchange).toHaveBeenLastCalledWith('phone');
		await expect.element(page.getByRole('radio', { name: 'Phone' })).toHaveFocus();
	});
});
