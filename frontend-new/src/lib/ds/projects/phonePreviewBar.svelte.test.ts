import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import PhonePreviewBar from './PhonePreviewBar.svelte';

// Flow / Mobile Build → step 5 (PsC5F) → Preview Bar GppXE: 40 high with a bottom stroke, the address
// in mono at 12, Reload and Open at 32, the device pair on a 32 track with 28 choices, the chosen one
// raised.
describe('PhonePreviewBar', () => {
	it('is the pen’s 40-high bar: the address in mono, Reload, Open and the device pair', async () => {
		const screen = await render(PhonePreviewBar, { address: 'warung-kopi-menu.trian.space', href: 'https://warung-kopi-menu.trian.space', device: 'desktop', onreload: () => {} });
		const bar = screen.container.querySelector('div.h-10') as HTMLElement;
		expect(bar.getBoundingClientRect().height).toBe(40);
		expect(getComputedStyle(bar).borderBottomWidth).toBe('1px');
		const address = page.getByText('warung-kopi-menu.trian.space');
		expect(getComputedStyle(address.element()).fontSize).toBe('12px');
		expect(getComputedStyle(address.element()).fontFamily).toMatch(/mono/i);
		expect(page.getByRole('button', { name: 'Reload' }).element().getBoundingClientRect().width).toBe(32);
		await expect.element(page.getByRole('link', { name: 'Open in a new tab' })).toHaveAttribute('href', 'https://warung-kopi-menu.trian.space');
		const desktop = page.getByRole('radio', { name: 'Desktop' });
		await expect.element(desktop).toHaveAttribute('aria-checked', 'true');
		expect(desktop.element().getBoundingClientRect().width).toBe(28);
		expect(getComputedStyle(desktop.element()).outlineStyle).toBe('solid');
	});

	it('leaves Open out without an address to open, reloads, and switches by tap and arrow keys', async () => {
		const onreload = vi.fn();
		const ondevice = vi.fn();
		await render(PhonePreviewBar, { address: 'changes not yet published', device: 'phone', onreload, ondevice });
		await expect.element(page.getByRole('link', { name: 'Open in a new tab' })).not.toBeInTheDocument();
		await page.getByRole('button', { name: 'Reload' }).click();
		expect(onreload).toHaveBeenCalledOnce();
		await page.getByRole('radio', { name: 'Desktop' }).click();
		expect(ondevice).toHaveBeenLastCalledWith('desktop');
		await userEvent.keyboard('{ArrowLeft}');
		expect(ondevice).toHaveBeenLastCalledWith('phone');
		await expect.element(page.getByRole('radio', { name: 'Phone' })).toHaveFocus();
	});
});
