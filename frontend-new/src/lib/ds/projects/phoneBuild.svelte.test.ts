import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import PhoneWorkspaceNav from './PhoneWorkspaceNav.svelte';
import PhoneAskBar from './PhoneAskBar.svelte';
import PhoneList from './PhoneList.svelte';
import PhoneListRow from './PhoneListRow.svelte';
import PhoneTile from './PhoneTile.svelte';
import PhoneNote from './PhoneNote.svelte';
import PresenceStrip from './PresenceStrip.svelte';
import PhoneFollowBar from './PhoneFollowBar.svelte';
import PhoneCodeLines from './PhoneCodeLines.svelte';
import PhoneTerminal from './PhoneTerminal.svelte';
import PhoneBuildShareSheet from './PhoneBuildShareSheet.svelte';
import PhoneToastRegion from '../overlays/PhoneToastRegion.svelte';
import { toasts } from '../overlays/toasts.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Flow / Mobile Build: a project's workspace on a phone (contract docs/ux/work/phone-build).

afterEach(() => {
	document.body.innerHTML = '';
	toasts.clear();
});
const box = (element: Element) => element.getBoundingClientRect();

describe('PhoneWorkspaceNav', () => {
	const base = { project: 'warung-kopi-menu', status: 'live · deploy #2', live: true, onback: () => {}, onmenu: () => {} };

	it('is 52 tall with back and ··· on 44 targets, the project in mono, and Up to date as a quiet state', async () => {
		await page.viewport(390, 844);
		await render(PhoneWorkspaceNav, { ...base, pill: { kind: 'idle' } });
		const nav = page.getByRole('banner').element();
		expect(box(nav).height).toBe(52);
		expect(box(page.getByRole('button', { name: 'Back' }).element()).width).toBe(44);
		expect(box(page.getByRole('button', { name: 'Project options' }).element()).width).toBe(44);
		expect(getComputedStyle(page.getByRole('heading', { level: 1 }).element()).fontFamily).toMatch(/Mono/i);
		await expect.element(page.getByText('Up to date')).toBeVisible();
		await expect.element(page.getByRole('button', { name: /Up to date/ })).not.toBeInTheDocument();
	});

	it('publishes from the mint pill with the count, retries from the danger one, and shows the time while publishing', async () => {
		const onpublish = vi.fn();
		const { rerender } = await render(PhoneWorkspaceNav, { ...base, pill: { kind: 'ready', count: 3, onpublish } });
		const ready = page.getByRole('button', { name: 'Publish · 3' });
		expect(getComputedStyle(ready.element()).backgroundColor).toBe(tokenBackground('nm-accent'));
		await ready.click();
		expect(onpublish).toHaveBeenCalledOnce();
		const onretry = vi.fn();
		await rerender({ pill: { kind: 'failed', onretry } });
		const failed = page.getByRole('button', { name: 'Failed · Retry' });
		expect(getComputedStyle(failed.element()).color).toBe(tokenColor('nm-danger'));
		await failed.click();
		expect(onretry).toHaveBeenCalledOnce();
		await rerender({ pill: { kind: 'publishing', elapsed: '0:24' } });
		await expect.element(page.getByText('Publishing… 0:24')).toBeVisible();
	});

	it('gives a member the ask as an icon with its name, and a viewer the state with nothing to press', async () => {
		const onask = vi.fn();
		const { rerender } = await render(PhoneWorkspaceNav, { ...base, pill: { kind: 'ask', onask } });
		await page.getByRole('button', { name: 'Ask to publish' }).click();
		expect(onask).toHaveBeenCalledOnce();
		const onopen = vi.fn();
		await rerender({ pill: { kind: 'asked', onopen } });
		await page.getByRole('button', { name: 'Asked to publish' }).click();
		expect(onopen).toHaveBeenCalledOnce();
		await rerender({ pill: { kind: 'failed' } });
		await expect.element(page.getByText('Failed', { exact: true })).toBeVisible();
		await expect.element(page.getByRole('button', { name: /Failed/ })).not.toBeInTheDocument();
	});
});

describe('PhoneAskBar', () => {
	it('floats at 52 with Koda and the pen’s “Ask Koda for a change…”, one tap to the chat', async () => {
		await page.viewport(390, 844);
		const onask = vi.fn();
		await render(PhoneAskBar, { onask });
		const bar = page.getByRole('button', { name: 'Ask Koda for a change…' });
		expect(box(bar.element()).height).toBe(52);
		await bar.click();
		expect(onask).toHaveBeenCalledOnce();
	});
});

describe('PhoneList and PhoneListRow', () => {
	it('lists rows in one card, the number in mono, and a row that opens is a button with its chevron', async () => {
		await page.viewport(390, 844);
		const onclick = vi.fn();
		const end = createRawSnippet(() => ({ render: () => '<span>Live</span>' }));
		await render(PhoneList, {
			label: 'Deployments',
			children: createRawSnippet(() => ({ render: () => '<li data-row>x</li>' })),
		});
		expect(page.getByRole('list', { name: 'Deployments' }).element()).toBeTruthy();
		document.body.innerHTML = '';
		await render(PhoneListRow, { lead: '#2', title: 'Deploy #2', meta: 'Today 08:42 · by Mario', end });
		expect(getComputedStyle(page.getByText('#2', { exact: true }).element()).fontFamily).toMatch(/Mono/i);
		await expect.element(page.getByText('Live')).toBeVisible();
		await expect.element(page.getByRole('button')).not.toBeInTheDocument();
		document.body.innerHTML = '';
		await render(PhoneListRow, { title: 'index.html', mono: true, onclick });
		const row = page.getByRole('button', { name: 'index.html' });
		expect(box(row.element()).height).toBeGreaterThanOrEqual(52);
		await row.click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});

describe('PhoneTile and PhoneNote', () => {
	it('shows a number at 22 with what it counts, and says Not measured rather than 0', async () => {
		await render(PhoneTile, { label: 'Requests', value: '128', note: 'today', notMeasured: 'Not measured' });
		expect(getComputedStyle(page.getByText('128').element()).fontSize).toBe('22px');
		document.body.innerHTML = '';
		await render(PhoneTile, { label: 'Errors', value: null, notMeasured: 'Not measured' });
		await expect.element(page.getByText('Not measured')).toBeVisible();
		await expect.element(page.getByText('0', { exact: true })).not.toBeInTheDocument();
		document.body.innerHTML = '';
		await render(PhoneNote, { text: 'Read-only here. Ask Koda to change columns.', boxed: true });
		expect(getComputedStyle(page.getByText('Read-only here.', { exact: false }).element()).backgroundColor).toBe(tokenBackground('nm-surface-2'));
	});
});

describe('PresenceStrip and PhoneFollowBar', () => {
	it('names who’s here and where, opens the list, and follows', async () => {
		const onopen = vi.fn();
		const onfollow = vi.fn();
		await render(PresenceStrip, { people: [{ name: 'Sari', initials: 'SW', peer: 1, where: 'Code' }], onopen, onfollow });
		await page.getByRole('button', { name: /Sari is here · Code/ }).click();
		expect(onopen).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: 'Follow Sari' }).click();
		expect(onfollow).toHaveBeenCalledOnce();
	});

	it('says whom you follow in their colour, 40 tall, and Stop ends it', async () => {
		await page.viewport(390, 844);
		const onstop = vi.fn();
		await render(PhoneFollowBar, { name: 'Sari', peer: 1, where: 'Code', onstop });
		const bar = page.getByRole('status').element();
		expect([box(bar).height, getComputedStyle(bar).backgroundColor]).toEqual([40, tokenBackground('nm-peer-1')]);
		await page.getByRole('button', { name: 'Stop following' }).click();
		expect(onstop).toHaveBeenCalledOnce();
	});
});

describe('PhoneCodeLines', () => {
	const lines = [[{ text: '<h1>Warung</h1>' }], [{ text: '<p>Kopi susu</p>' }], [{ text: '<p>Es kopi</p>' }]];

	it('picks lines with the keyboard (Enter, Shift+↓) and asks about them; Escape lets go', async () => {
		const onask = vi.fn();
		await render(PhoneCodeLines, { lines, label: 'index.html', onask });
		await expect.element(page.getByRole('button', { name: /Ask Koda about lines/ })).not.toBeInTheDocument();
		(page.getByRole('button', { name: 'Line 1' }).element() as HTMLElement).focus();
		await userEvent.keyboard('{Enter}');
		await userEvent.keyboard('{Shift>}{ArrowDown}{/Shift}');
		await expect.element(page.getByRole('button', { name: 'Line 2' })).toHaveAttribute('aria-pressed', 'true');
		await page.getByRole('button', { name: 'Ask Koda about lines 1–2' }).click();
		expect(onask).toHaveBeenCalledWith(1, 2);
		(page.getByRole('button', { name: 'Line 2' }).element() as HTMLElement).focus();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('button', { name: /Ask Koda about lines/ })).not.toBeInTheDocument();
	});

	it('wraps a long line at 390 instead of scrolling sideways', async () => {
		await page.viewport(390, 844);
		await render(PhoneCodeLines, { lines: [[{ text: 'x'.repeat(400) }]], label: 'long.ts', onask: () => {} });
		const list = page.getByRole('list', { name: 'long.ts' }).element();
		expect(list.scrollWidth).toBeLessThanOrEqual(box(list).width);
	});
});

describe('PhoneTerminal', () => {
	it('shows the log, wrapped, and a failure with its fix at the foot', async () => {
		const onfix = vi.fn();
		await render(PhoneTerminal, { label: 'dev server · sandbox', lines: [{ kind: 'command', text: '$ npm run dev' }, { kind: 'fail', text: 'error: port in use' }], failure: { title: 'The preview didn’t start', detail: 'Terminal shows what went wrong.', fix: 'Ask Koda to fix', onfix } });
		expect(getComputedStyle(page.getByText('error: port in use').element()).color).toBe(tokenColor('nm-danger'));
		await expect.element(page.getByRole('alert')).toHaveTextContent('The preview didn’t start');
		await page.getByRole('button', { name: 'Ask Koda to fix' }).click();
		expect(onfix).toHaveBeenCalledOnce();
	});
});

describe('PhoneBuildShareSheet', () => {
	it('copies the address and sends it to WhatsApp, Telegram and Mail; Build together opens invites', async () => {
		const oninvite = vi.fn();
		await render(PhoneBuildShareSheet, { open: true, project: 'warung-kopi-menu', url: 'https://warung-kopi-menu.trian.space', oninvite });
		await expect.element(page.getByRole('dialog', { name: 'Share warung-kopi-menu' })).toBeVisible();
		await expect.element(page.getByRole('link', { name: 'WhatsApp' })).toHaveAttribute('href', 'https://wa.me/?text=https%3A%2F%2Fwarung-kopi-menu.trian.space');
		await expect.element(page.getByRole('link', { name: 'Telegram' })).toHaveAttribute('href', 'https://t.me/share/url?url=https%3A%2F%2Fwarung-kopi-menu.trian.space');
		await expect.element(page.getByRole('link', { name: 'Mail' })).toHaveAttribute('href', 'mailto:?body=https%3A%2F%2Fwarung-kopi-menu.trian.space');
		await page.getByRole('button', { name: /Build together/ }).click();
		expect(oninvite).toHaveBeenCalledOnce();
	});

	it('leads a site with its card and whether it’s on show', async () => {
		const onvisibility = vi.fn();
		await render(PhoneBuildShareSheet, { open: true, project: 'studio-rupa', url: 'https://studio-rupa.trian.space', card: { title: 'studio-rupa', description: 'Portfolio Studio Rupa' }, visibility: 'unlisted', onvisibility });
		await expect.element(page.getByText('Portfolio Studio Rupa')).toBeVisible();
		await page.getByRole('radio', { name: 'Public' }).click();
		expect(onvisibility).toHaveBeenCalledWith('public');
	});
});

describe('PhoneToastRegion', () => {
	it('shows the app’s toasts at the top as phone toasts, with their action', async () => {
		await page.viewport(390, 844);
		const onclick = vi.fn();
		await render(PhoneToastRegion);
		toasts.show({ kind: 'success', title: 'Deploy #2 is live', action: { label: 'Roll back to #1', onclick } });
		const toast = page.getByRole('status').filter({ hasText: 'Deploy #2 is live' });
		await expect.element(toast).toBeVisible();
		expect(box(toast.element()).top).toBeLessThan(80);
		await page.getByRole('button', { name: 'Roll back to #1' }).click();
		expect(onclick).toHaveBeenCalledOnce();
		await expect.element(toast).not.toBeInTheDocument();
	});
});
