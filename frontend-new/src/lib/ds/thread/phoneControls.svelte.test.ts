import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { Copy, ThumbsUp } from '@lucide/svelte';
import SheetTiles from '../overlays/SheetTiles.svelte';
import PhoneMessageEditor from './PhoneMessageEditor.svelte';
import PhoneDislikeSheet from './PhoneDislikeSheet.svelte';
import PhoneShareSheet from './PhoneShareSheet.svelte';
import LiftedSheetFixture from './LiftedSheetFixture.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// The phone's message controls (Flow / Mobile Chat — Message controls, Like & dislike, Share &
// delete): a reply's quick tiles, editing your message in place, the dislike sheet, the share sheet.

afterEach(() => {
	document.body.innerHTML = '';
});
const box = (element: Element) => element.getBoundingClientRect();

describe('SheetTiles', () => {
	it('lays the quick actions out as equal tiles, one that’s on in accent', async () => {
		await page.viewport(390, 844);
		const oncopy = vi.fn();
		await render(SheetTiles, { tiles: [{ label: 'Good', icon: ThumbsUp, pressed: true, onclick: () => {} }, { label: 'Copy', icon: Copy, onclick: oncopy }] });
		const good = page.getByRole('button', { name: 'Good' }).element();
		const copy = page.getByRole('button', { name: 'Copy' }).element();
		expect([box(good).width, box(good).height, getComputedStyle(good).color, getComputedStyle(good).backgroundColor]).toEqual([box(copy).width, 68, tokenColor('nm-accent'), tokenBackground('nm-surface-2')]);
		await page.getByRole('button', { name: 'Copy' }).click();
		expect(oncopy).toHaveBeenCalledOnce();
	});
});

describe('PhoneMessageEditor', () => {
	it('edits in place at 16, saying which version it saves as; Send only once it changed', async () => {
		const onsave = vi.fn();
		const oncancel = vi.fn();
		await render(PhoneMessageEditor, { value: 'How much on food in October?', version: 'saves as version 2', onsave, oncancel });
		const field = page.getByRole('textbox', { name: 'Your message' });
		await expect.element(field).toHaveFocus();
		expect(getComputedStyle(field.element()).fontSize).toBe('16px');
		await expect.element(page.getByRole('button', { name: 'Send' })).toBeDisabled();
		await field.fill('How much on food in October, without office lunches?');
		await userEvent.keyboard('{Enter}');
		expect(onsave).toHaveBeenCalledWith('How much on food in October, without office lunches?');
		await page.getByRole('button', { name: 'Cancel' }).click();
		expect(oncancel).toHaveBeenCalledOnce();
	});
});

describe('PhoneDislikeSheet', () => {
	it('picks reasons as chips, takes a note and whether to include the chat, and sends', async () => {
		await page.viewport(390, 844);
		const onsend = vi.fn();
		await render(PhoneDislikeSheet, { open: true, onsend });
		const accurate = page.getByRole('button', { name: 'Not accurate' });
		await accurate.click();
		await expect.element(accurate).toHaveAttribute('aria-pressed', 'true');
		expect(getComputedStyle(accurate.element()).backgroundColor).toBe(tokenBackground('nm-text'));
		await page.getByRole('textbox', { name: 'What should it have said?' }).fill('It counted office lunches.');
		await page.getByRole('switch', { name: 'Include the whole chat' }).click();
		await page.getByRole('button', { name: 'Send feedback' }).click();
		expect(onsend).toHaveBeenCalledWith({ reasons: ['inaccurate'], note: 'It counted office lunches.', includeChat: true });
	});
});

describe('PhoneShareSheet', () => {
	it('shares a reply or the whole chat, masked or not, and copies the link from the accent tile', async () => {
		await page.viewport(390, 844);
		const onchange = vi.fn();
		const oncopy = vi.fn();
		await render(PhoneShareSheet, {
			open: true,
			excerpt: 'Rp ••• on food in October, up 20% from September.',
			chatMessages: 6,
			url: 'https://nomi.app/shared/abc',
			expiry: 'Expires in 7 days',
			settings: { scope: 'reply', mask: true, anyone: true },
			onchange,
			oncopy,
		});
		await expect.element(page.getByRole('radio', { name: 'This reply' })).toHaveAttribute('aria-checked', 'true');
		await page.getByRole('radio', { name: 'Whole chat · 6' }).click();
		expect(onchange).toHaveBeenLastCalledWith({ scope: 'chat', mask: true, anyone: true });
		await page.getByRole('switch', { name: 'Hide amounts and names' }).click();
		expect(onchange).toHaveBeenLastCalledWith({ scope: 'reply', mask: false, anyone: true });
		await page.getByRole('button', { name: 'Copy link' }).click();
		expect(oncopy).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('link', { name: 'WhatsApp' })).toHaveAttribute('href', 'https://wa.me/?text=https%3A%2F%2Fnomi.app%2Fshared%2Fabc');
	});
});

describe('Your message lifted over its sheet (Message controls nzT7f)', () => {
	it('holds the bubble right-aligned 12 above the sheet, 16 from the edge, stroked border-strong with the lift shadow', async () => {
		await page.viewport(390, 844);
		await render(LiftedSheetFixture);
		const bubble = page.getByText('How much did I spend on food in October? @Dana').element();
		const sheet = document.querySelector('[data-sheet]')!;
		expect([Math.round(box(sheet).top - box(bubble).bottom), Math.round(390 - box(bubble).right), Math.round(box(bubble).width)]).toEqual([12, 16, 278]);
		const look = getComputedStyle(bubble);
		expect([look.outlineColor, look.backgroundColor, look.borderRadius, look.boxShadow.includes('6px 18px')]).toEqual([tokenColor('nm-border-strong'), tokenBackground('nm-surface-2'), '18px', true]);
		// A tap beside the bubble closes the sheet, as the scrim does.
		(document.querySelector('[data-lifted]') as HTMLElement).click();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});
});
