import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ProjectNav from './ProjectNav.svelte';
import WorkspaceSwitch from './WorkspaceSwitch.svelte';
import AskBar from './AskBar.svelte';
import BuildPlanCard from './BuildPlanCard.svelte';
import BuildProgress from './BuildProgress.svelte';
import LiveCard from './LiveCard.svelte';
import DnsRecord from './DnsRecord.svelte';
import TeammateBubble from './TeammateBubble.svelte';
import AskQueue from './AskQueue.svelte';
import ChangeReady from './ChangeReady.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Build → Build on mobile: one column at 390pt. Chat and Preview switch with a segmented
// control; every project tool is a bottom sheet; the live app gets the full screen with Koda one
// tap away.

const sari = { name: 'Sari', initials: 'SW', peer: 1 as const };

describe('Project bar', () => {
	it('names the project, whether it is live, and the publish state, on 44pt targets', async () => {
		const onback = vi.fn();
		await render(ProjectNav, { project: 'warung-kopi-menu', status: 'live · deploy #2', publish: 'idle', onback, onmenu: () => {} });
		expect(getComputedStyle(page.getByText('warung-kopi-menu').element()).fontFamily).toContain('Geist Mono');
		await expect.element(page.getByRole('button', { name: 'Up to date' })).toBeDisabled();
		const back = page.getByRole('button', { name: 'Back' });
		expect(back.element().getBoundingClientRect().width).toBe(44);
		await back.click();
		expect(onback).toHaveBeenCalledOnce();
	});

	it('switches between Chat and Preview, with changes waiting counted', async () => {
		const onchange = vi.fn();
		await render(WorkspaceSwitch, { value: 'chat', previewCount: 3, onchange });
		await expect.element(page.getByRole('radio', { name: /Chat/ })).toHaveAttribute('aria-checked', 'true');
		await expect.element(page.getByRole('radio', { name: /Preview/ })).toHaveTextContent('3');
		await page.getByRole('radio', { name: /Preview/ }).click();
		expect(onchange).toHaveBeenCalledWith('preview');
	});

	it('floats an ask bar over the live app', async () => {
		const onask = vi.fn();
		const onvoice = vi.fn();
		await render(AskBar, { onask, onvoice });
		await page.getByRole('button', { name: 'Ask Koda about this screen' }).click();
		await page.getByRole('button', { name: 'Record a voice note' }).click();
		expect(onask).toHaveBeenCalledOnce();
		expect(onvoice).toHaveBeenCalledOnce();
	});
});

describe('Building on a phone', () => {
	it('shows the plan with one big Build it and the other choices under it', async () => {
		const onbuild = vi.fn();
		const onkeep = vi.fn();
		await render(BuildPlanCard, {
			project: 'warung-kopi-menu',
			summary: 'Menu and pre-order page for Warung Kopi Bu Siti',
			steps: ['Set up the project and a small database', 'Test it on a phone'],
			outcome: 'A live link customers can open today. The code is yours.',
			cost: 'About 5 min · included in your free plan',
			onbuild,
			onaskfirst: () => {},
			onkeep,
		});
		expect(getComputedStyle(page.getByRole('heading', { name: /Menu and pre-order page/ }).element()).fontSize).toBe('18px');
		await page.getByRole('button', { name: 'Build it' }).click();
		await page.getByRole('button', { name: 'Keep planning' }).click();
		expect(onbuild).toHaveBeenCalledOnce();
		expect(onkeep).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('button', { name: 'Ask before it goes live' })).toBeVisible();
	});

	it('shows the real step while building and that you can close the app', async () => {
		const onstop = vi.fn();
		const { container } = await render(BuildProgress, { project: 'warung-kopi-menu', elapsed: '01:12', step: 'writing src/routes/menu/+page.svelte', current: 2, total: 6, label: 'menu page', eta: '~3 min left', onstop });
		await expect.element(page.getByText('Building warung-kopi-menu')).toBeVisible();
		await expect.element(page.getByText('Step 2 of 6 · menu page · ~3 min left')).toBeVisible();
		await expect.element(page.getByText("You can close the app. We'll notify you when it's live.")).toBeVisible();
		expect(getComputedStyle(container.querySelectorAll('[data-step]')[1]).backgroundColor).toBe(tokenBackground('nm-text'));
		await page.getByRole('button', { name: 'Stop' }).click();
		expect(onstop).toHaveBeenCalledOnce();
	});

	it('gives the live link to copy, open or share', async () => {
		await render(LiveCard, { meta: 'deploy #1 · 08:18', url: 'https://warung-kopi-menu-3f9a12.trian.space', summary: 'Built in 5m 40s', onshare: () => {} });
		await expect.element(page.getByRole('link', { name: 'Open' })).toHaveAttribute('href', 'https://warung-kopi-menu-3f9a12.trian.space');
		await expect.element(page.getByRole('button', { name: 'Copy link' })).toBeVisible();
		await expect.element(page.getByText('Live', { exact: true })).toBeVisible();
	});

	it('shows one domain record per card, copied in one tap', async () => {
		await render(DnsRecord, { type: 'A', name: '@', value: '76.76.21.21', state: 'waiting' });
		expect(getComputedStyle(page.getByText('waiting').element()).color).toBe(tokenColor('nm-warning'));
		expect(getComputedStyle(page.getByText('76.76.21.21').element()).fontFamily).toContain('Geist Mono');
		await expect.element(page.getByRole('button', { name: 'Copy 76.76.21.21' })).toBeVisible();
	});
});

describe('Together on a phone', () => {
	it('shows a teammate’s message in their colour at 16pt', async () => {
		await render(TeammateBubble, { who: sari, time: '09:14', to: 'Koda', text: 'Can we add a Makanan section?' });
		await expect.element(page.getByText('09:14 · to Koda')).toBeVisible();
		const bubble = page.getByText('Can we add a Makanan section?').element();
		expect(getComputedStyle(bubble).fontSize).toBe('16px');
		expect(getComputedStyle(bubble).backgroundColor).toBe(tokenBackground('nm-peer-1-bg'));
	});

	it('queues asks and lets the next one go first', async () => {
		const onfirst = vi.fn();
		await render(AskQueue, {
			asks: [
				{ text: 'Make the Pre-order button green', who: { name: 'Mario', initials: 'MP' }, state: 'running', meta: 'Mario · running · 00:21' },
				{ text: 'Add a Makanan section with nasi uduk', who: sari, state: 'next', meta: 'Sari · next' },
			],
			onfirst,
		});
		await expect.element(page.getByText('2 asks in line')).toBeVisible();
		await page.getByRole('button', { name: 'Let Sari go first' }).click();
		expect(onfirst).toHaveBeenCalledWith(1);
	});

	it('asks to publish a teammate’s ready change', async () => {
		const onpublish = vi.fn();
		await render(ChangeReady, { who: sari, meta: 'Makanan section · 2 files · tests pass', onpublish, onreview: () => {} });
		await expect.element(page.getByText("Sari's change is ready")).toBeVisible();
		await page.getByRole('button', { name: 'Publish' }).click();
		expect(onpublish).toHaveBeenCalledOnce();
	});
});
