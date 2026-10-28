import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import AppShellFixture from './AppShellFixture.svelte';
import WorkspaceShellFixture from './WorkspaceShellFixture.svelte';
import FocusShell from './FocusShell.svelte';

// Layout shells: columns are fixed, content fills. All screens are 1512 × 862 at 1×; phones get
// the mobile layouts from the same shells.

afterEach(async () => {
	await page.viewport(1512, 862);
});

describe('AppShell', () => {
	it('lays out sidebar, top bar, content and crew panel on a desktop', async () => {
		await page.viewport(1512, 862);
		await render(AppShellFixture);
		await expect.element(page.getByText('Sidebar content')).toBeVisible();
		await expect.element(page.getByText('Crew panel')).toBeVisible();
		await expect.element(page.getByRole('main')).toHaveTextContent('Page content');
		expect(getComputedStyle(page.getByText('Morning, Mario.').element().closest('header')!).height).toBe('52px');
		expect(page.getByRole('button', { name: 'Open navigation' }).elements()).toHaveLength(0);
	});

	it('puts the sidebar in a drawer and the panel away on a phone', async () => {
		await page.viewport(390, 844);
		await render(AppShellFixture);
		await expect.element(page.getByText('Sidebar content')).not.toBeVisible();
		await expect.element(page.getByText('Crew panel')).not.toBeVisible();
		await page.getByRole('button', { name: 'Open navigation' }).click();
		const drawer = page.getByRole('dialog', { name: 'Navigation' });
		await expect.element(drawer).toBeVisible();
		await expect.element(drawer).toHaveTextContent('Sidebar content');
	});
});

describe('WorkspaceShell', () => {
	it('puts the agent chat (440) beside the pane on a desktop', async () => {
		await page.viewport(1512, 862);
		await render(WorkspaceShellFixture);
		const chat = page.getByRole('region', { name: 'Chat' }).element();
		expect(getComputedStyle(chat).width).toBe('440px');
		await expect.element(page.getByText('Preview pane')).toBeVisible();
		expect(getComputedStyle(page.getByText('warung-kopi-menu').element().closest('header')!).height).toBe('52px');
	});

	it('switches between chat and pane at the top on a phone', async () => {
		await page.viewport(390, 844);
		await render(WorkspaceShellFixture);
		await expect.element(page.getByText('Agent chat')).toBeVisible();
		await expect.element(page.getByText('Preview pane')).not.toBeVisible();
		await page.getByRole('radio', { name: 'Preview' }).click();
		await expect.element(page.getByText('Preview pane')).toBeVisible();
		await expect.element(page.getByText('Agent chat')).not.toBeVisible();
	});
});

describe('FocusShell', () => {
	it('centres one column, 480 or 720 wide', async () => {
		await page.viewport(1512, 862);
		const narrow = await render(FocusShell, { width: 'narrow' });
		expect(getComputedStyle(narrow.container.querySelector('[data-column]')!).maxWidth).toBe('480px');
		const wide = await render(FocusShell, { width: 'wide' });
		expect(getComputedStyle(wide.container.querySelector('[data-column]')!).maxWidth).toBe('720px');
	});
});
