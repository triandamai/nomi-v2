import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { FolderOpen, House, Inbox, MessageCircle, Plus, SquarePen, User } from '@lucide/svelte';
import TabBar from './TabBar.svelte';
import Fab from './Fab.svelte';
import LargeTitle from './LargeTitle.svelte';
import BottomSheetFixture from '../overlays/BottomSheetFixture.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// The phone's own chrome (Flow / Mobile Chat — Start & answer → Chats): the tab bar, the large
// title, the floating New chat, and the bottom sheet the phone uses in place of popovers.

afterEach(() => {
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const tabs = [
	{ href: '/', label: 'Home', icon: House },
	{ href: '/chats', label: 'Chats', icon: MessageCircle, current: true },
	{ href: '/projects', label: 'Projects', icon: FolderOpen },
	{ href: '/inbox', label: 'Inbox', icon: Inbox },
	{ href: '/settings', label: 'You', icon: User },
];

describe('TabBar', () => {
	it('is the pen’s 50pt bar of five equal tabs over a top stroke, the current one in text and 600', async () => {
		await page.viewport(390, 844);
		await render(TabBar, { label: 'Main', tabs });
		const nav = page.getByRole('navigation', { name: 'Main' }).element();
		const style = getComputedStyle(nav);
		// The stroke is drawn inside in the pen: 1px of the 8 above the tabs is the stroke.
		expect([box(nav).width, box(nav).height, style.paddingTop, style.borderTopWidth, style.paddingLeft, style.borderTopColor]).toEqual([390, 50, '7px', '1px', '12px', tokenColor('nm-border')]);
		const links = [...nav.querySelectorAll('a')];
		expect(links.map((link) => Math.round(box(link).width))).toEqual([73, 73, 73, 73, 73]);
		const current = page.getByRole('link', { name: 'Chats' });
		await expect.element(current).toHaveAttribute('aria-current', 'page');
		expect([getComputedStyle(current.element()).color, getComputedStyle(current.element().querySelector('span:last-child')!).fontWeight]).toEqual([tokenColor('nm-text'), '600']);
		expect(getComputedStyle(page.getByRole('link', { name: 'Home' }).element()).color).toBe(tokenColor('nm-text-3'));
		const icon = current.element().querySelector('svg')!;
		expect([box(icon).width, getComputedStyle(current.element().querySelector('span:last-child')!).fontSize]).toEqual([22, '10px']);
	});
});

describe('Fab', () => {
	it('is a 56 mint circle with its icon, floating, and leads to a new chat', async () => {
		await render(Fab, { label: 'New chat', icon: SquarePen, iconSize: 22, href: '/chat/new' });
		const link = page.getByRole('link', { name: 'New chat' });
		await expect.element(link).toHaveAttribute('href', '/chat/new');
		const style = getComputedStyle(link.element());
		expect([box(link.element()).width, box(link.element()).height, style.backgroundColor]).toEqual([56, 56, tokenBackground('nm-accent')]);
		expect(parseFloat(style.borderRadius)).toBeGreaterThanOrEqual(28);
		expect(style.boxShadow).not.toBe('none');
	});
});

describe('LargeTitle', () => {
	it('is the tab’s title at 32, 600, padded [4,20,10,20]', async () => {
		await render(LargeTitle, { title: 'Chats' });
		const heading = page.getByRole('heading', { level: 1, name: 'Chats' }).element();
		const style = getComputedStyle(heading);
		expect([style.fontSize, style.fontWeight, style.letterSpacing]).toEqual(['32px', '600', '-0.6px']);
		const wrap = getComputedStyle(heading.parentElement!);
		expect([wrap.paddingTop, wrap.paddingLeft, wrap.paddingBottom]).toEqual(['4px', '20px', '10px']);
	});
});

describe('BottomSheet', () => {
	it('rises from the bottom over the scrim: surface, 22 corners on top, the header and its rows', async () => {
		await page.viewport(390, 844);
		const onpick = vi.fn();
		await render(BottomSheetFixture, { onpick });
		const sheet = page.getByRole('dialog', { name: 'Food spending in October' });
		await expect.element(sheet).toBeVisible();
		const panel = sheet.element().querySelector<HTMLElement>('[data-sheet]')!;
		const style = getComputedStyle(panel);
		expect([style.backgroundColor, style.borderTopLeftRadius, style.borderBottomLeftRadius, style.borderTopColor]).toEqual([tokenBackground('nm-surface'), '22px', '0px', tokenColor('nm-border-strong')]);
		expect([Math.round(box(panel).width), Math.round(box(panel).bottom)]).toEqual([390, 844]);
		await page.getByRole('button', { name: 'Pin to top' }).click();
		expect(onpick).toHaveBeenCalledWith('pin');
	});

	it('closes on Escape, the close button, or a tap on the scrim', async () => {
		await page.viewport(390, 844);
		await render(BottomSheetFixture, {});
		const sheet = page.getByRole('dialog', { name: 'Food spending in October' });
		await userEvent.keyboard('{Escape}');
		await expect.element(sheet).not.toBeInTheDocument();
		await page.getByRole('button', { name: 'Open' }).click();
		await page.getByRole('button', { name: 'Close' }).click();
		await expect.element(sheet).not.toBeInTheDocument();
		await page.getByRole('button', { name: 'Open' }).click();
		await userEvent.click(document.body, { position: { x: 195, y: 40 } });
		await expect.element(sheet).not.toBeInTheDocument();
	});
});

describe('Fab as a button', () => {
	it('opens what it opens, with the 24 plus by default (New project)', async () => {
		const onclick = vi.fn();
		await render(Fab, { label: 'New project', icon: Plus, onclick });
		const button = page.getByRole('button', { name: 'New project' });
		expect(button.element().querySelector('svg')!.getBoundingClientRect().width).toBe(24);
		await button.click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});
