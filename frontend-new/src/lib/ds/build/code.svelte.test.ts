import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import CodeView from './CodeView.svelte';
import Terminal from './Terminal.svelte';
import DeviceBar from './DeviceBar.svelte';
import PhoneFrame from './PhoneFrame.svelte';
import CodeQuote from './CodeQuote.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Build → Code, terminal & phone: the preview pane's other two tabs and the phone switch. Code and
// Terminal are read-only: what Koda wrote and what Koda ran. Changes always go through the chat.

afterEach(() => {
	vi.restoreAllMocks();
});

const tree = [
	{ name: 'src', depth: 0, kind: 'folder' as const },
	{ name: 'routes', depth: 1, kind: 'folder' as const },
	{ name: '+page.svelte', depth: 2, kind: 'file' as const, active: true },
	{ name: 'db/schema.sql', depth: 1, kind: 'database' as const },
];
const lines = [
	[{ text: '<script ', kind: 'type' as const }, { text: 'lang', kind: 'fn' as const }, { text: '=', kind: 'punct' as const }, { text: '"ts"', kind: 'string' as const }, { text: '>' }],
	[{ text: '  import ', kind: 'keyword' as const }, { text: '{ menu }' }],
	[],
	[{ text: '<PickupSlots ', kind: 'type' as const }, { text: 'every', kind: 'fn' as const }, { text: '={' }, { text: '15', kind: 'number' as const }, { text: '} />' }],
];

describe('CodeView', () => {
	it('shows the file tree and the read-only file with what Koda last changed', async () => {
		await page.viewport(1280, 800);
		await render(CodeView, { branch: 'main@9d2e41a', tree, path: 'src/routes/menu/+page.svelte', meta: 'read-only · edited by Koda 08:41', lines, onask: () => {}, onasklines: () => {} });
		await expect.element(page.getByText('main@9d2e41a', { exact: false })).toBeVisible();
		expect(getComputedStyle(page.getByText('Files', { exact: true }).element()).textTransform).toBe('uppercase');
		const active = page.getByRole('treeitem', { name: '+page.svelte' });
		await expect.element(active).toHaveAttribute('aria-selected', 'true');
		expect(getComputedStyle(page.getByText('"ts"').element()).color).toBe(tokenColor('nm-code-string'));
		expect(getComputedStyle(page.getByText('=', { exact: true }).element()).color).toBe(tokenColor('nm-text-2'));
		await expect.element(page.getByText('Read-only. Ask Koda in the chat to change anything; every change is a commit you can roll back.')).toBeVisible();
	});

	it('selects lines by their numbers and asks Koda about them', async () => {
		const onasklines = vi.fn();
		const onask = vi.fn();
		await render(CodeView, { tree, path: 'a.svelte', lines, onask, onasklines });
		await page.getByRole('button', { name: 'Ask Koda about this file' }).click();
		expect(onask).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: 'Line 2' }).click();
		await userEvent.keyboard('{Shift>}');
		await page.getByRole('button', { name: 'Line 4' }).click();
		await userEvent.keyboard('{/Shift}');
		await expect.element(page.getByRole('button', { name: 'Line 3' })).toHaveAttribute('aria-pressed', 'true');
		const row = page.getByRole('button', { name: 'Line 3' }).element().closest('[data-line]')!;
		expect(getComputedStyle(row).backgroundColor).toBe(tokenBackground('nm-accent-bg'));
		await page.getByRole('button', { name: 'Ask Koda about lines 2–4' }).click();
		expect(onasklines).toHaveBeenCalledWith(2, 4);
	});
});

describe('CodeView with a selection', () => {
	it('can start with lines picked', async () => {
		await render(CodeView, { tree, path: 'a.svelte', lines, selection: [3, 4], onask: () => {}, onasklines: () => {} });
		await expect.element(page.getByRole('button', { name: 'Ask Koda about lines 3–4' })).toBeVisible();
	});
});

describe('Terminal', () => {
	const run = [
		{ kind: 'command' as const, text: '$ npm run build', time: '08:41:02' },
		{ kind: 'out' as const, text: 'vite v5.4.2 building for production…' },
		{ kind: 'ok' as const, text: '✓ built in 9.6s' },
	];

	it('shows what Koda ran, in order, read-only, and copies the log', async () => {
		const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();
		await render(Terminal, { run: 'run #2 · publish deploy #2', lines: run });
		expect(getComputedStyle(page.getByText('✓ built in 9.6s').element()).color).toBe(tokenColor('nm-accent'));
		expect(getComputedStyle(page.getByText('08:41:02').element()).color).toBe(tokenColor('nm-text-3'));
		await expect.element(page.getByText('Koda runs the commands. Want something run? Ask in the chat.')).toBeVisible();
		await page.getByRole('button', { name: 'Copy log' }).click();
		expect(write).toHaveBeenCalledWith('$ npm run build\nvite v5.4.2 building for production…\n✓ built in 9.6s');
	});

	it('sums a failure up in one line, with what went live and a fix', async () => {
		const onfix = vi.fn();
		await render(Terminal, {
			run: 'run #3',
			lines: [{ kind: 'fail' as const, text: '✗ tests/order.test.ts (4) · 1 failed' }, { kind: 'warn' as const, text: 'deploy skipped: tests failed' }],
			failure: { title: '1 test failed: no weekend pickup slots after 11:00', detail: 'Nothing went live. Customers still see deploy #2.', onfix },
		});
		const failed = page.getByText('✗ tests/order.test.ts (4) · 1 failed').element().parentElement!;
		expect(getComputedStyle(failed).backgroundColor).toBe(tokenBackground('nm-danger-bg'));
		expect(getComputedStyle(page.getByText('deploy skipped: tests failed').element()).color).toBe(tokenColor('nm-warning'));
		await page.getByRole('button', { name: 'Ask Koda to fix' }).click();
		expect(onfix).toHaveBeenCalledOnce();
	});
});

describe('Phone preview', () => {
	it('picks a device and opens the app on your phone', async () => {
		const ondevice = vi.fn();
		const onphone = vi.fn();
		await render(DeviceBar, { device: 'iPhone 15 · 393 × 852', ondevice, onphone });
		await page.getByRole('button', { name: /iPhone 15 · 393 × 852/ }).click();
		await page.getByRole('button', { name: 'Open on your phone' }).click();
		expect(ondevice).toHaveBeenCalledOnce();
		expect(onphone).toHaveBeenCalledOnce();
	});

	it('frames the app at phone size, light, behind a bezel', async () => {
		const { container } = await render(PhoneFrame, { src: 'about:blank', title: 'warung-kopi-menu preview' });
		const frame = container.querySelector('[data-phone]')!;
		// The pen's 330×640 screen with its 8px bezel drawn outside it.
		expect(frame.clientWidth).toBe(330);
		expect(frame.clientHeight).toBe(640);
		expect(frame.getBoundingClientRect().width).toBe(346);
		expect(getComputedStyle(frame).borderTopLeftRadius).toBe('44px');
		expect(frame.getAttribute('data-theme')).toBe('light');
		await expect.element(page.getByTitle('warung-kopi-menu preview')).toBeInTheDocument();
	});
});

describe('CodeQuote', () => {
	it('quotes the selected lines in the composer and can be removed', async () => {
		const onremove = vi.fn();
		await render(CodeQuote, { file: '+page.svelte', range: '14–15', snippet: '<PickupSlots from="07:00" to="11:00" />', onremove });
		await expect.element(page.getByText('+page.svelte · lines 14–15')).toBeVisible();
		expect(getComputedStyle(page.getByText('<PickupSlots', { exact: false }).element()).color).toBe(tokenColor('nm-text-3'));
		await page.getByRole('button', { name: 'Remove quote' }).click();
		expect(onremove).toHaveBeenCalledOnce();
	});
});
