import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { RefreshCw, Send, Trash2 } from '@lucide/svelte';
import NavBar from './NavBar.svelte';
import Composer from './Composer.svelte';
import UserMessage from './UserMessage.svelte';
import Byline from './Byline.svelte';
import ActionBar from './ActionBar.svelte';
import PhoneButton from './PhoneButton.svelte';
import StatList from './StatList.svelte';
import ErrorCard from './ErrorCard.svelte';
import SheetHeader from './SheetHeader.svelte';
import SheetRow from './SheetRow.svelte';
import PhoneToast from './PhoneToast.svelte';
import Progress from './Progress.svelte';
import PhoneOption from './PhoneOption.svelte';
import ApprovalCard from './ApprovalCard.svelte';
import ChatRow from './ChatRow.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Chat responses → Mobile chat: native patterns at 390 pt. Sheets instead of popovers, 44 pt
// targets, 16 pt body text, the composer above the keyboard, banners at the top.

const box = (element: Element) => element.getBoundingClientRect();
const style = (element: Element) => getComputedStyle(element);

describe('Shell', () => {
	it('has a nav bar with 44pt back and menu, the title and who answers', async () => {
		const onback = vi.fn();
		await render(NavBar, { title: 'Food spending in October', who: { agent: 'dana', name: 'Dana' }, onback, onmenu: () => {} });
		const back = page.getByRole('button', { name: 'Back' });
		expect(box(back.element()).width).toBe(44);
		expect(box(page.getByRole('button', { name: 'Chat options' }).element()).height).toBe(44);
		const title = page.getByRole('heading', { name: 'Food spending in October' });
		expect(style(title.element()).fontSize).toBe('16px');
		expect(style(title.element()).fontWeight).toBe('600');
		await back.click();
		expect(onback).toHaveBeenCalledOnce();
	});

	it('composes in 16pt (no zoom on focus), sends only with text, and stops a reply', async () => {
		const onsend = vi.fn();
		const { rerender } = await render(Composer, { onsend, onattach: () => {}, onrecord: () => {} });
		const field = page.getByRole('textbox', { name: 'Message' });
		expect(style(field.element()).fontSize).toBe('16px');
		expect(page.getByRole('button', { name: 'Send' }).elements()).toHaveLength(0);
		await field.fill('Berapa pengeluaran makan?');
		await page.getByRole('button', { name: 'Send' }).click();
		expect(onsend).toHaveBeenCalledWith('Berapa pengeluaran makan?');
		expect(box(page.getByRole('button', { name: 'Attach a file' }).element()).width).toBe(38);
		const onstop = vi.fn();
		await rerender({ onsend, onstop, streaming: true });
		await page.getByRole('button', { name: 'Stop reply' }).click();
		expect(onstop).toHaveBeenCalledOnce();
	});
});

describe('Messages', () => {
	it('shows your message in an 18pt-round bubble at 16pt', async () => {
		await render(UserMessage, { text: 'How much did I spend on food in October?' });
		const bubble = page.getByText('How much did I spend on food in October?').element();
		expect(style(bubble).fontSize).toBe('16px');
		expect(style(bubble).borderTopLeftRadius).toBe('18px');
		expect(style(bubble).backgroundColor).toBe(tokenBackground('nm-surface-2'));
	});

	it('says when it was not sent, with a retry', async () => {
		const onretry = vi.fn();
		await render(UserMessage, { text: 'Hi', status: 'failed', onretry });
		expect(style(page.getByText('not sent').element()).color).toBe(tokenColor('nm-danger'));
		await page.getByRole('button', { name: 'Retry' }).click();
		expect(onretry).toHaveBeenCalledOnce();
	});

	it('names who answered at 14pt, with the time in mono', async () => {
		await render(Byline, { agent: 'dana', name: 'Dana', meta: '14:02' });
		expect(style(page.getByText('Dana').element()).fontSize).toBe('14px');
		expect(style(page.getByText('14:02').element()).fontFamily).toContain('Geist Mono');
	});

	it('puts the reply actions on 38pt round buttons', async () => {
		const onmore = vi.fn();
		await render(ActionBar, { text: 'x', onlike: () => {}, ondislike: () => {}, onshare: () => {}, onmore });
		const more = page.getByRole('button', { name: 'More' });
		expect(box(more.element()).width).toBe(38);
		expect(style(more.element()).borderTopLeftRadius).not.toBe('0px');
		await more.click();
		expect(onmore).toHaveBeenCalledOnce();
	});

	it('lists comparable rows as a stat list, deltas toned', async () => {
		await render(StatList, {
			title: 'Food by week',
			sub: 'Oct · Rp',
			rows: [{ label: '1–7 Oct', value: '480rb', delta: { value: '+12%', tone: 'bad' }, detail: 'Groceries 312rb · GoFood 168rb' }],
			total: { label: 'October', value: 'Rp 2,1 jt', delta: { value: '+20%', tone: 'bad' } },
		});
		expect(page.getByRole('listitem').elements()).toHaveLength(2);
		expect(style(page.getByText('+12%').element()).color).toBe(tokenColor('nm-danger'));
		expect(style(page.getByText('480rb').element()).fontFamily).toContain('Geist Mono');
	});
});

describe('Buttons and cards', () => {
	it('makes full-width 46pt buttons in three kinds', async () => {
		await render(PhoneButton, { label: 'Approve & send', icon: Send, onclick: () => {} });
		const button = page.getByRole('button', { name: 'Approve & send' });
		expect(box(button.element()).height).toBe(46);
		expect(style(button.element()).backgroundColor).toBe(tokenBackground('nm-accent'));
		expect(style(button.element()).fontSize).toBe('16px');
		const danger = await render(PhoneButton, { label: 'Delete reply', icon: Trash2, variant: 'danger', onclick: () => {} });
		expect(style(danger.container.querySelector('button')!).color).toBe(tokenColor('nm-danger'));
	});

	it('quotes the provider in the error card and stacks the fixes', async () => {
		const onprimary = vi.fn();
		await render(ErrorCard, {
			title: 'Out of credits',
			code: '402',
			provider: 'DeepSeek',
			reason: 'Insufficient Balance',
			meta: 'deepseek-flash · key ••a91f · req_9d41e0b2',
			primary: { label: 'Switch to Nomi credits', onclick: onprimary },
			secondary: { label: 'Add credits on DeepSeek', href: 'https://platform.deepseek.com', external: true },
		});
		await expect.element(page.getByText('DeepSeek said')).toBeVisible();
		expect(style(page.getByText('Insufficient Balance').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('button', { name: 'Switch to Nomi credits' }).click();
		expect(onprimary).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('link', { name: 'Add credits on DeepSeek' })).toHaveAttribute('target', '_blank');
	});
});

describe('Sheets and banners', () => {
	it('heads a sheet with a grabber, the title and a close', async () => {
		const onclose = vi.fn();
		await render(SheetHeader, { title: 'Reply', subtitle: 'Dana · 14:02', onclose });
		expect(style(page.getByRole('heading', { name: 'Reply' }).element()).fontSize).toBe('17px');
		await page.getByRole('button', { name: 'Close' }).click();
		expect(onclose).toHaveBeenCalledOnce();
	});

	it('lists sheet actions as 52pt rows', async () => {
		const onclick = vi.fn();
		await render(SheetRow, { icon: RefreshCw, label: 'Regenerate', onclick });
		const row = page.getByRole('button', { name: 'Regenerate' });
		expect(box(row.element()).height).toBe(52);
		await row.click();
		expect(onclick).toHaveBeenCalledOnce();
	});

	it('toasts with a 16pt-round card and one action', async () => {
		const onclick = vi.fn();
		await render(PhoneToast, { kind: 'success', title: 'Link copied', detail: 'Expires in 7 days', action: { label: 'Undo', onclick } });
		const toast = page.getByRole('status');
		expect(style(toast.element()).borderTopLeftRadius).toBe('16px');
		await page.getByRole('button', { name: 'Undo' }).click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});

describe('Decide on mobile', () => {
	it('shows how far along a set of questions is', async () => {
		const { container } = await render(Progress, { current: 1, total: 4 });
		await expect.element(page.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '1');
		const segments = [...container.querySelectorAll('[data-segment]')];
		expect(segments).toHaveLength(4);
		expect(style(segments[0]).backgroundColor).toBe(tokenBackground('nm-text'));
		expect(style(segments[1]).backgroundColor).toBe(tokenBackground('nm-border-strong'));
	});

	it('picks an option with a big round radio', async () => {
		const onselect = vi.fn();
		await render(PhoneOption, { name: 'trip', key: 1, title: 'Whoosh high-speed train', detail: '45 min from Halim', checked: false, onselect });
		const radio = page.getByRole('radio', { name: /Whoosh high-speed train/ });
		expect(box(radio.element()).width).toBe(22);
		await radio.click();
		expect(onselect).toHaveBeenCalledOnce();
	});

	it('approves on a phone with the message previewed and two full buttons', async () => {
		const onapprove = vi.fn();
		const onopen = vi.fn();
		await render(ApprovalCard, {
			agent: 'tara',
			title: 'Tara wants to send an email',
			to: 'bu.sri.kosmawar@gmail.com',
			subject: 'Transfer sewa November sudah masuk',
			excerpt: 'Selamat siang Bu Sri…',
			attachment: 'bukti-transfer-nov.jpg',
			approveLabel: 'Approve',
			rejectLabel: "Don't send",
			onapprove,
			onreject: () => {},
			onopen,
		});
		await expect.element(page.getByText('to bu.sri.kosmawar@gmail.com')).toBeVisible();
		await page.getByRole('button', { name: 'Approve' }).click();
		expect(onapprove).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: /Tara wants to send an email/ }).click();
		expect(onopen).toHaveBeenCalledOnce();
	});
});

describe('Chats list', () => {
	it('shows a chat with who, when, the last line and unread', async () => {
		await render(ChatRow, { href: '/chat/1', agent: 'dana', title: 'Food spending in October', time: '14:02', preview: 'Dana: Rp 2,1 jt on food…', unread: true });
		const link = page.getByRole('link', { name: /Food spending in October/ });
		await expect.element(link).toHaveAttribute('href', '/chat/1');
		await expect.element(page.getByText('Unread')).toBeInTheDocument();
		await userEvent.hover(link);
	});
});
