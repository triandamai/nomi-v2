import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import BuildPlan from './BuildPlan.svelte';
import BuildRunning from './BuildRunning.svelte';
import BuildDone from './BuildDone.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Build → Build estimate: shown before anything runs, like a Claude plan: the steps, what you'll
// have, and one clear choice. Cost is a quiet line, never the headline. It collapses into the
// thread once the build starts.

afterEach(() => {
	vi.restoreAllMocks();
});

const plan = {
	project: 'warung-kopi-menu',
	summary: 'Menu and pre-order page for Warung Kopi Bu Siti',
	steps: ['Set up the project and a small database for orders', 'Menu page with Bu Siti’s items, prices and photos', 'Put it online at a link you can share'],
	outcome: 'When it’s done: a live link customers can open today. The code is yours to export.',
	options: [
		{ title: 'Build it', detail: 'Koda builds it and puts it online.', recommended: true },
		{ title: 'Build it, ask before it goes live', detail: 'Everything is built in the preview first.' },
	],
	cost: 'About 5 min · included in your free plan',
};

describe('BuildPlan', () => {
	it('shows the plan before anything runs: project, steps, outcome, then one choice and a quiet cost', async () => {
		const onrun = vi.fn();
		await render(BuildPlan, { ...plan, onrun });
		await expect.element(page.getByText('Here’s what Koda will build')).toBeVisible();
		expect(page.getByRole('listitem').elements()).toHaveLength(3);
		expect(getComputedStyle(page.getByText('warung-kopi-menu').element()).fontFamily).toContain('Geist Mono');
		const outcome = page.getByText(plan.outcome).element();
		expect(getComputedStyle(outcome).backgroundColor).toBe(tokenBackground('nm-accent-bg'));
		await expect.element(page.getByRole('radio', { name: /Build it Recommended/ })).toBeChecked();
		expect(getComputedStyle(page.getByText(plan.cost).element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('button', { name: 'Build it' }).click();
		expect(onrun).toHaveBeenCalledWith({ choice: 0 });
	});

	it('keeps planning with what to change, saying so on the button', async () => {
		const onrun = vi.fn();
		await render(BuildPlan, { ...plan, onrun });
		await page.getByRole('textbox', { name: 'Tell Koda what to change…' }).fill('Add a WhatsApp button');
		await page.getByRole('button', { name: 'Send changes' }).click();
		expect(onrun).toHaveBeenCalledWith({ keepPlanning: 'Add a WhatsApp button' });
	});
});

describe('BuildRunning', () => {
	it('shows the real step it is on, its time, a stop, and steps done so far', async () => {
		const onstop = vi.fn();
		const { container } = await render(BuildRunning, {
			started: '08:13',
			step: 'writing src/routes/menu/+page.svelte',
			elapsed: '01:12',
			current: 2,
			total: 6,
			label: 'menu page',
			eta: '~3 min left',
			onstop,
		});
		await expect.element(page.getByText('running · started 08:13')).toBeVisible();
		await expect.element(page.getByText('Koda is building')).toBeVisible();
		await expect.element(page.getByText('Step 2 of 6 · menu page')).toBeVisible();
		const bars = [...container.querySelectorAll('[data-step]')];
		expect(bars).toHaveLength(6);
		expect(getComputedStyle(bars[0]).backgroundColor).toBe(tokenBackground('nm-accent'));
		expect(getComputedStyle(bars[1]).backgroundColor).toBe(tokenBackground('nm-text-2'));
		expect(getComputedStyle(bars[2]).backgroundColor).toBe(tokenBackground('nm-border-strong'));
		await page.getByRole('button', { name: 'Stop' }).click();
		expect(onstop).toHaveBeenCalledOnce();
	});
});

describe('BuildDone', () => {
	it('gives the live link, what it actually took, and the next step', async () => {
		const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();
		const ondetails = vi.fn();
		await render(BuildDone, {
			meta: 'deploy #1 · 08:18',
			url: 'https://warung-kopi-menu-3f9a12.trian.space',
			summary: 'Built in 5m 40s · used your free allowance',
			ondetails,
			onshare: () => {},
		});
		await expect.element(page.getByText('Live', { exact: true })).toBeVisible();
		const link = page.getByText('warung-kopi-menu-3f9a12.trian.space');
		expect(getComputedStyle(link.element()).fontFamily).toContain('Geist Mono');
		await expect.element(page.getByRole('link', { name: 'Open' })).toHaveAttribute('href', 'https://warung-kopi-menu-3f9a12.trian.space');
		await page.getByRole('button', { name: 'Copy link' }).click();
		expect(write).toHaveBeenCalledWith('https://warung-kopi-menu-3f9a12.trian.space');
		await page.getByRole('button', { name: 'Details' }).click();
		expect(ondetails).toHaveBeenCalledOnce();
		await expect.element(page.getByText('Changes after this need Publish')).toBeVisible();
		expect(getComputedStyle(page.getByRole('button', { name: 'Share' }).element()).borderTopColor).toBe(tokenColor('nm-border-strong'));
	});
});
