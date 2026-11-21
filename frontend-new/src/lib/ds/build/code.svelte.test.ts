import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import CodeView, { FILE_DRAG_TYPE } from './CodeView.svelte';
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
		await page.getByRole('button', { name: 'Line 2', exact: true }).click();
		await userEvent.keyboard('{Shift>}');
		await page.getByRole('button', { name: 'Line 4', exact: true }).click();
		await userEvent.keyboard('{/Shift}');
		await expect.element(page.getByRole('button', { name: 'Line 3', exact: true })).toHaveAttribute('aria-pressed', 'true');
		const row = page.getByRole('button', { name: 'Line 3', exact: true }).element().closest('[data-line]')!;
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
		// The pen's 330×640 phone; its 8px bezel is drawn on the edge (as the pen exports it: an
		// outline offset -4) and takes no room.
		expect(frame.getBoundingClientRect().width).toBe(330);
		expect(frame.getBoundingClientRect().height).toBe(640);
		expect(getComputedStyle(frame).outlineWidth).toBe('8px');
		expect(getComputedStyle(frame).outlineOffset).toBe('-4px');
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

describe('CodeView folders, line menu and drag', () => {
	const pathed = [
		{ name: 'src', depth: 0, kind: 'folder' as const, path: 'src' },
		{ name: 'routes', depth: 1, kind: 'folder' as const, path: 'src/routes' },
		{ name: '+page.svelte', depth: 2, kind: 'file' as const, path: 'src/routes/+page.svelte', active: true },
		{ name: 'app.css', depth: 1, kind: 'file' as const, path: 'src/app.css' },
		{ name: 'README.md', depth: 0, kind: 'text' as const, path: 'README.md' },
	];

	it('closes and opens a folder, hiding what is inside it', async () => {
		const onopen = vi.fn();
		await page.viewport(1280, 800);
		await render(CodeView, { tree: pathed, path: 'src/routes/+page.svelte', lines, onopen, onask: () => {}, onasklines: () => {} });
		const src = page.getByRole('treeitem', { name: 'src', exact: true });
		await expect.element(src).toHaveAttribute('aria-expanded', 'true');
		await src.click();
		await expect.element(src).toHaveAttribute('aria-expanded', 'false');
		await expect.element(page.getByRole('treeitem', { name: 'app.css' })).not.toBeInTheDocument();
		await expect.element(page.getByRole('treeitem', { name: 'README.md' })).toBeVisible();
		expect(onopen).not.toHaveBeenCalled();
		(src.element() as HTMLElement).focus();
		await userEvent.keyboard('{ArrowRight}');
		await expect.element(page.getByRole('treeitem', { name: 'app.css' })).toBeVisible();
		await page.getByRole('treeitem', { name: 'app.css' }).click();
		expect(onopen).toHaveBeenCalledWith(3);
	});

	it('drags a file out of the tree with its path', async () => {
		await page.viewport(1280, 800);
		await render(CodeView, { tree: pathed, path: 'src/routes/+page.svelte', lines, onask: () => {}, onasklines: () => {} });
		const file = page.getByRole('treeitem', { name: 'app.css' }).element() as HTMLElement;
		expect(file.draggable).toBe(true);
		expect((page.getByRole('treeitem', { name: 'src', exact: true }).element() as HTMLElement).draggable).toBe(false);
		const data = new DataTransfer();
		file.dispatchEvent(new DragEvent('dragstart', { bubbles: true, dataTransfer: data }));
		expect(data.getData(FILE_DRAG_TYPE)).toBe('src/app.css');
	});

	it('opens a line’s menu from its gutter “+” and asks Koda about the line', async () => {
		const onasklines = vi.fn();
		await render(CodeView, { tree, path: 'a.svelte', lines, onask: () => {}, onasklines });
		const plus = page.getByRole('button', { name: 'Actions for line 2' });
		expect(getComputedStyle(plus.element()).opacity).toBe('0');
		await page.getByRole('button', { name: 'Line 2', exact: true }).hover();
		await expect.poll(() => getComputedStyle(plus.element()).opacity).toBe('1');
		await plus.click();
		await expect.element(page.getByRole('menu')).toBeVisible();
		await expect.element(page.getByRole('menuitem', { name: 'Copy line' })).toBeVisible();
		await page.getByRole('menuitem', { name: 'Ask Koda about this line' }).click();
		expect(onasklines).toHaveBeenCalledWith(2, 2);
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
	});

	it('closes the line menu with Escape, focus back on its “+”', async () => {
		await render(CodeView, { tree, path: 'a.svelte', lines, onask: () => {}, onasklines: () => {} });
		await page.getByRole('button', { name: 'Actions for line 3' }).click();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
		expect(document.activeElement?.getAttribute('aria-label')).toBe('Actions for line 3');
	});
});

describe('Terminal in a pane', () => {
	it('fills the pane’s height even with nothing printed', async () => {
		const host = document.createElement('div');
		host.style.height = '700px';
		document.body.append(host);
		await render(Terminal, { target: host, props: { run: 'sandbox', lines: [], fill: true } });
		expect(host.querySelector('section')!.getBoundingClientRect().height).toBe(700);
		host.remove();
	});
});
