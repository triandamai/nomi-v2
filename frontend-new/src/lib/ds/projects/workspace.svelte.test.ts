import { afterEach, describe, expect, it, vi } from 'vitest';
import { createRawSnippet } from 'svelte';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { Activity, Database, History, Link, Settings } from '@lucide/svelte';
import ProjectBar from './ProjectBar.svelte';
import PaneTabs from './PaneTabs.svelte';
import ChatPaneHeader from './ChatPaneHeader.svelte';
import ProjectWorkspaceFixture from './ProjectWorkspaceFixture.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// The project workspace's chrome (Flow / Build — Sign in → live URL o882RC, Screen / Building
// H4DIH), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const nav = [
	{ id: 'deployments', label: 'Deployments', icon: History },
	{ id: 'domain', label: 'Domain', icon: Link },
	{ id: 'monitor', label: 'Monitor', icon: Activity },
	{ id: 'database', label: 'Database', icon: Database },
	{ id: 'settings', label: 'Settings', icon: Settings },
];
const bar = {
	backHref: '/projects',
	backLabel: 'Back to projects',
	agent: 'koda' as const,
	name: 'warung-kopi-menu',
	status: { label: 'ready', tone: 'warning' as const },
	space: 'trian.space',
	url: 'https://warung-kopi-menu.trian.space',
	nav,
	shareLabel: 'Share',
	exportLabel: 'Export repo',
	initials: 'MP',
	navLabel: 'Project tools',
};

describe('ProjectBar', () => {
	it('is 52 high over its line, 16 in: back, the name with its status, where it lives', async () => {
		await page.viewport(1512, 862);
		await render(ProjectBar, bar);
		const header = page.getByRole('banner').element();
		expect([box(header).height, getComputedStyle(header).borderBottomColor]).toEqual([52, tokenColor('nm-border')]);
		const back = page.getByRole('link', { name: 'Back to projects' }).element();
		expect([box(back).left - box(header).left, box(back).width, back.getAttribute('href')]).toEqual([16, 28, '/projects']);
		await expect.element(page.getByText('warung-kopi-menu', { exact: true })).toBeVisible();
		const dot = page.getByText('ready').element().previousElementSibling!;
		expect(getComputedStyle(dot).backgroundColor).toBe(tokenBackground('nm-warning'));
		const address = page.getByRole('link', { name: /warung-kopi-menu\.trian\.space/ }).element();
		expect([address.getAttribute('href'), address.getAttribute('target')]).toEqual(['https://warung-kopi-menu.trian.space', '_blank']);
	});

	it('opens each tool, marks the open one, and shares and exports', async () => {
		const onnav = vi.fn();
		const onshare = vi.fn();
		const onexport = vi.fn();
		await render(ProjectBar, { ...bar, active: 'domain', onnav, onshare, onexport });
		const tools = page.getByRole('navigation', { name: 'Project tools' });
		await tools.getByRole('button', { name: 'Deployments' }).click();
		expect(onnav).toHaveBeenCalledWith('deployments');
		await expect.element(tools.getByRole('button', { name: 'Domain' })).toHaveAttribute('aria-pressed', 'true');
		await page.getByRole('button', { name: 'Share' }).click();
		await page.getByRole('button', { name: 'Export repo' }).click();
		expect([onshare.mock.calls.length, onexport.mock.calls.length]).toEqual([1, 1]);
	});

	it('shows who’s here in place of the person’s own face, when others are', async () => {
		const presence = createRawSnippet(() => ({ render: () => '<button type="button">2 people here</button>' }));
		await render(ProjectBar, { ...bar, presence });
		await expect.element(page.getByRole('button', { name: '2 people here' })).toBeVisible();
		expect(page.getByText('MP', { exact: true }).elements()).toHaveLength(0);
	});
});

describe('PaneTabs', () => {
	const tabs = { labels: { views: 'View', preview: 'Preview', code: 'Code', terminal: 'Terminal', device: 'Device', desktop: 'Desktop', phone: 'Phone', reload: 'Reload', open: 'Open in a new tab', close: 'Close the preview' } };

	it('is 44 high: the view switch, the live address with its deploy, the device switch', async () => {
		await page.viewport(1512, 862);
		await render(PaneTabs, { ...tabs, address: { live: 'live', url: 'https://warung.trian.space', shown: 'warung.trian.space', deploy: 'deploy #1' } });
		const strip = page.getByText('warung.trian.space').element().closest('[data-part="tabs"]')!;
		expect([box(strip).height, getComputedStyle(strip).borderBottomColor]).toEqual([44, tokenColor('nm-border')]);
		const preview = page.getByRole('radio', { name: 'Preview' }).element();
		expect([box(preview).height, getComputedStyle(preview).backgroundColor, preview.getAttribute('aria-checked')]).toEqual([26, tokenBackground('nm-bg'), 'true']);
		await expect.element(page.getByText('deploy #1')).toBeVisible();
		await expect.element(page.getByRole('link', { name: 'Open in a new tab' })).toHaveAttribute('href', 'https://warung.trian.space');
	});

	it('switches view and device, and reloads', async () => {
		const onreload = vi.fn();
		await render(PaneTabs, { ...tabs, address: { url: '/projects/p1/preview/', shown: 'preview' }, onreload });
		await page.getByRole('radio', { name: 'Code' }).click();
		await expect.element(page.getByRole('radio', { name: 'Code' })).toHaveAttribute('aria-checked', 'true');
		await page.getByRole('radio', { name: 'Phone' }).click();
		await expect.element(page.getByRole('radio', { name: 'Phone' })).toHaveAttribute('aria-checked', 'true');
		await page.getByRole('button', { name: 'Reload' }).click();
		expect(onreload).toHaveBeenCalledOnce();
		expect(page.getByText('live').elements()).toHaveLength(0);
	});

	it('shows a preview with no address of its own without Open, but still reloads', async () => {
		const onreload = vi.fn();
		await render(PaneTabs, { ...tabs, address: { shown: 'changes not yet published' }, onreload });
		await expect.element(page.getByText('changes not yet published')).toBeVisible();
		expect(page.getByRole('link', { name: 'Open in a new tab' }).elements()).toHaveLength(0);
		await page.getByRole('button', { name: 'Reload' }).click();
		expect(onreload).toHaveBeenCalledOnce();
	});

	it('offers only the views asked for', async () => {
		await render(PaneTabs, { ...tabs, views: ['preview', 'code'] });
		expect(page.getByRole('radio', { name: 'Terminal' }).elements()).toHaveLength(0);
	});
});

describe('ChatPaneHeader', () => {
	it('is 44 high, [0,12,0,20]: Chat, who’s working in mono', async () => {
		await page.viewport(1512, 862);
		await render(ChatPaneHeader, { title: 'Chat', crew: ['koda', 'nomi'], note: 'koda working with nomi' });
		const header = page.getByText('Chat', { exact: true }).element().parentElement!;
		expect([box(header).height, getComputedStyle(header).paddingLeft, getComputedStyle(header).borderBottomColor]).toEqual([44, '20px', tokenColor('nm-border')]);
		expect(getComputedStyle(page.getByText('koda working with nomi').element()).fontFamily).toMatch(/Mono/);
	});
});

describe('ProjectWorkspace', () => {
	it('puts the agent chat (520, as Flow / Build draws it) beside the pane, the resize handle between', async () => {
		await page.viewport(1512, 862);
		await render(ProjectWorkspaceFixture);
		const chat = page.getByRole('region', { name: 'Chat' }).element();
		expect(getComputedStyle(chat).width).toBe('520px');
		await expect.element(page.getByRole('separator')).toBeVisible();
		await expect.element(page.getByText('Preview pane')).toBeVisible();
	});

	it('switches between chat and pane at the top on a phone', async () => {
		await page.viewport(390, 844);
		await render(ProjectWorkspaceFixture);
		await expect.element(page.getByText('Agent chat')).toBeVisible();
		await page.getByRole('radio', { name: 'Preview' }).click();
		await expect.element(page.getByText('Preview pane')).toBeVisible();
		await expect.element(page.getByText('Agent chat')).not.toBeVisible();
		await page.viewport(1512, 862);
	});
});
