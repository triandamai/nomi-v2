import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { Copy, Share, ThumbsDown, ThumbsUp } from '@lucide/svelte';
import SheetTiles from '../overlays/SheetTiles.svelte';
import ActionSheet from '../overlays/ActionSheet.svelte';
import BottomSheetFixture from '../overlays/BottomSheetFixture.svelte';
import PhoneChatNavBar from './PhoneChatNavBar.svelte';
import PhoneMessageBubble from './PhoneMessageBubble.svelte';
import PhoneStarters from './PhoneStarters.svelte';
import PhoneComposer from './PhoneComposer.svelte';
import PhoneMessageEditor from './PhoneMessageEditor.svelte';
import PhoneDislikeSheet from './PhoneDislikeSheet.svelte';
import PhoneShareSheet from './PhoneShareSheet.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// The phone chat screens against the pen (docs/ux/work/phone-fidelity/50-build-verify.md): each
// value here is the one the audit read from the pen, item by item.

afterEach(() => {
	document.body.innerHTML = '';
});
const box = (element: Element) => element.getBoundingClientRect();
const style = (element: Element) => getComputedStyle(element);

describe('a chat’s chrome on a phone', () => {
	it('wears who answers at 20 in the nav bar (item 4)', async () => {
		await page.viewport(390, 844);
		await render(PhoneChatNavBar, { title: 'Food spending in October', who: { agent: 'dana', name: 'Dana' }, onback: () => {}, onmenu: () => {} });
		const who = document.querySelector('[data-who]')!;
		expect(box(who).height).toBe(20);
		expect(box(who.querySelector('svg')!).width).toBe(20);
	});

	it('wraps your message at 250, so the bubble is 278 (item 6)', async () => {
		await page.viewport(390, 844);
		await render(PhoneMessageBubble, { text: 'How much did I spend on food in October, without the office lunches and the coffee runs?' });
		const bubble = page.getByText(/How much did I spend/).element();
		expect(box(bubble).width).toBe(278);
	});

	it('keeps the pill’s border-strong while the person types (item 5)', async () => {
		await page.viewport(390, 844);
		await render(PhoneComposer, { placeholder: 'Message', onsend: () => {} });
		const field = page.getByRole('textbox', { name: 'Message' });
		await field.click();
		await expect.element(field).toHaveFocus();
		expect(style(field.element().parentElement!).borderTopColor).toBe(tokenColor('nm-border-strong'));
	});

	it('draws New chat’s starters at 15, a long one on two lines in a 58 row (item 8)', async () => {
		await page.viewport(390, 844);
		const onpick = vi.fn();
		await render(PhoneStarters, {
			items: [
				{ agent: 'koda', label: 'Build a menu page for my warung', route: '→ koda' },
				{ agent: 'maya', label: 'Turn a photo of my price list into an order form', route: '→ maya' },
			],
			onpick,
		});
		const list = document.querySelector('ul')!;
		list.style.width = '358px';
		const long = page.getByRole('button', { name: /Turn a photo/ }).element();
		const label = [...long.querySelectorAll('span')].find((one) => one.textContent === 'Turn a photo of my price list into an order form')!;
		expect([style(label).fontSize, style(label).color]).toEqual(['15px', tokenColor('nm-text')]);
		expect(box(long).height).toBe(58);
		await page.getByRole('button', { name: /Turn a photo/ }).click();
		expect(onpick).toHaveBeenCalledWith(1);
	});
});

describe('a reply’s sheet on a phone', () => {
	it('lays the quick tiles out 68 tall, the icon 6 over its name in text-2, 10 under the row (item 15)', async () => {
		await page.viewport(390, 844);
		await render(SheetTiles, {
			tiles: [
				{ label: 'Good', icon: ThumbsUp, onclick: () => {} },
				{ label: 'Bad', icon: ThumbsDown, onclick: () => {} },
				{ label: 'Copy', icon: Copy, onclick: () => {} },
				{ label: 'Share', icon: Share, onclick: () => {} },
			],
		});
		const good = page.getByRole('button', { name: 'Good' }).element();
		const name = good.querySelector('span')!;
		expect(box(good).height).toBe(68);
		expect(box(name).top - box(good.querySelector('svg')!).bottom).toBe(6);
		expect(style(name).color).toBe(tokenColor('nm-text-2'));
		expect(style(good.parentElement!).paddingBottom).toBe('10px');
	});
});

describe('the dislike sheet on a phone', () => {
	it('offers the pen’s six reasons at 15, unselected in text-2 (items 19, 20)', async () => {
		await page.viewport(390, 844);
		await render(PhoneDislikeSheet, { open: true, onsend: () => {} });
		const reasons = page.getByRole('group', { name: 'What went wrong?' }).getByRole('button');
		await expect.element(reasons.first()).toBeVisible();
		expect(reasons.elements().map((one) => one.textContent?.trim())).toEqual(['Not accurate', 'Wrong crew member', "Didn't do what I asked", 'Too long', 'Out of date', 'Other']);
		const first = reasons.first().element();
		expect([style(first).fontSize, style(first).color]).toEqual(['15px', tokenColor('nm-text-2')]);
	});

	it('draws the note on bg, 12 round, 12 in, 84 tall; the body 14 apart; the switch’s label at 15 (items 21, 22)', async () => {
		await page.viewport(390, 844);
		await render(PhoneDislikeSheet, { open: true, onsend: () => {} });
		const note = page.getByRole('textbox', { name: 'What should it have said?' }).element();
		await expect.element(page.getByRole('textbox', { name: 'What should it have said?' })).toBeVisible();
		expect([style(note).backgroundColor, style(note).borderTopLeftRadius, box(note).height]).toEqual([tokenBackground('nm-bg'), '12px', 84]);
		expect(parseFloat(style(note).paddingTop) + parseFloat(style(note).borderTopWidth)).toBe(12);
		expect(parseFloat(style(note).paddingLeft) + parseFloat(style(note).borderLeftWidth)).toBe(12);
		expect(style(note.parentElement!).rowGap).toBe('14px');
		expect(style(page.getByText('Include the whole chat', { exact: true }).element()).fontSize).toBe('15px');
	});
});

describe('the share sheet on a phone', () => {
	const props = {
		open: true,
		excerpt: 'Rp ••• on food in October, up 20% from September.',
		chatMessages: 6,
		url: 'https://nomi.app/shared/abc',
		expiry: 'Expires in 7 days',
		settings: { scope: 'reply' as const, mask: true, anyone: true },
		onchange: () => {},
		oncopy: () => {},
	};

	it('sends to five tiles 65 wide, 8 apart, the icon 22 and the name 11 (item 23)', async () => {
		await page.viewport(390, 844);
		Object.defineProperty(navigator, 'share', { value: () => Promise.resolve(), configurable: true });
		await render(PhoneShareSheet, props);
		const copy = page.getByRole('button', { name: 'Copy link' }).element();
		const whatsapp = page.getByRole('link', { name: 'WhatsApp' }).element();
		await expect.element(page.getByRole('button', { name: 'More' })).toBeVisible();
		expect(box(copy).width).toBe(65);
		expect(box(whatsapp).left - box(copy).right).toBe(8);
		expect(box(copy.querySelector('svg')!).width).toBe(22);
		expect(style(page.getByText('Telegram').element()).fontSize).toBe('11px');
		delete (navigator as { share?: unknown }).share;
	});

	it('says what the link is in the pen’s words, the footnote in lower case (items 24, 25)', async () => {
		await page.viewport(390, 844);
		await render(PhoneShareSheet, props);
		await expect.element(page.getByText('Read-only, no sign-in', { exact: true })).toBeVisible();
		const foot = page.getByRole('button', { name: 'Expires in 7 days' }).element();
		expect(style(foot).textTransform).toBe('lowercase');
	});

	it('shows the preview on bg in a border, 12 round and 12 in, at 14 / 1.45 in text-2 (item 26)', async () => {
		await page.viewport(390, 844);
		await render(PhoneShareSheet, props);
		const card = page.getByText(/on food in October/).element();
		await expect.element(page.getByText(/on food in October/)).toBeVisible();
		expect([style(card).backgroundColor, style(card).outlineColor, style(card).outlineWidth, style(card).borderTopLeftRadius, style(card).paddingTop, style(card).paddingLeft]).toEqual([
			tokenBackground('nm-bg'),
			tokenColor('nm-border'),
			'1px',
			'12px',
			'12px',
			'12px',
		]);
		expect([style(card).fontSize, style(card).lineHeight, style(card).color]).toEqual(['14px', '20px', tokenColor('nm-text-2')]);
	});

	it('picks what to share in a 40 segment, 3 in, 10 round; the chosen half on bg, 34 tall, 600 (item 27)', async () => {
		await page.viewport(390, 844);
		await render(PhoneShareSheet, props);
		const group = page.getByRole('radiogroup', { name: 'What to share' }).element();
		const chosen = page.getByRole('radio', { name: 'This reply' }).element();
		await expect.element(page.getByRole('radio', { name: 'This reply' })).toBeVisible();
		expect([box(group).height, style(group).paddingTop, style(group).borderTopLeftRadius, style(group).columnGap]).toEqual([40, '3px', '10px', '3px']);
		expect([box(chosen).height, style(chosen).backgroundColor, style(chosen).fontWeight]).toEqual([34, tokenBackground('nm-bg'), '600']);
	});

	it('spaces the body 12 apart, the switches’ labels at 15 (item 28)', async () => {
		await page.viewport(390, 844);
		await render(PhoneShareSheet, props);
		const label = page.getByText('Hide amounts and names', { exact: true }).element();
		await expect.element(page.getByText('Hide amounts and names', { exact: true })).toBeVisible();
		expect(style(label).fontSize).toBe('15px');
		expect(style(page.getByRole('radiogroup', { name: 'What to share' }).element().parentElement!).rowGap).toBe('12px');
	});
});

describe('confirming and editing on a phone', () => {
	it('sets the action sheet’s message at 1.4 (item 30)', async () => {
		await page.viewport(390, 844);
		await render(ActionSheet, { open: true, title: 'Delete this chat?', body: '14 messages and their cards are removed.', actions: [{ label: 'Delete chat', danger: true, onclick: () => {} }], cancel: 'Cancel' });
		await expect.element(page.getByText('14 messages and their cards are removed.')).toBeVisible();
		expect(style(page.getByText('14 messages and their cards are removed.').element()).lineHeight).toBe('18px');
	});

	it('opens a sheet without a focus ring on its close (item 33)', async () => {
		await page.viewport(390, 844);
		await render(BottomSheetFixture, {});
		const close = page.getByRole('button', { name: 'Close' });
		await expect.element(close).toBeVisible();
		await expect.element(close).not.toHaveFocus();
		expect(document.activeElement?.tagName).toBe('DIALOG');
	});

	it('draws Cancel and Send 34 tall, 14 in: Cancel 15 / 400 in a border-strong stroke, Send 15 / 500 with a 14 arrow; the box 16 round (items 34, 35)', async () => {
		await page.viewport(390, 844);
		await render(PhoneMessageEditor, { value: 'How much on food in October?', version: 'saves as version 2', onsave: () => {}, oncancel: () => {} });
		const cancel = page.getByRole('button', { name: 'Cancel' }).element();
		const send = page.getByRole('button', { name: 'Send' }).element();
		expect([box(cancel).height, box(send).height]).toEqual([34, 34]);
		expect([style(cancel).fontSize, style(cancel).fontWeight, style(cancel).color, style(cancel).borderTopColor, parseFloat(style(cancel).paddingLeft) + parseFloat(style(cancel).borderLeftWidth)]).toEqual([
			'15px',
			'400',
			tokenColor('nm-text'),
			tokenColor('nm-border-strong'),
			14,
		]);
		expect([style(send).fontSize, style(send).fontWeight, style(send).paddingLeft, box(send.querySelector('svg')!).width]).toEqual(['15px', '500', '14px', 14]);
		expect(style(cancel.closest('[data-editor]')!).borderTopLeftRadius).toBe('16px');
	});
});
