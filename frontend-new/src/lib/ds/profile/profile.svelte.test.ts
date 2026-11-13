import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ProfileHeader from './ProfileHeader.svelte';
import ProfileAvatar from './ProfileAvatar.svelte';
import StatStrip from './StatStrip.svelte';
import ActivityHeatmap from './ActivityHeatmap.svelte';
import UsageBars from './UsageBars.svelte';
import CrewUsage from './CrewUsage.svelte';
import ShowcaseCard from './ShowcaseCard.svelte';
import ShowcaseAdd from './ShowcaseAdd.svelte';
import SaveBar from './SaveBar.svelte';
import { tokenBackground, tokenColor } from '../test-utils';
import { heatmap } from '$lib/settings/profile';

// Settings › Profile's pieces (App / Settings / Profile U9Gnrg), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

const tiles = [
	{ value: '186.4K', label: 'Lifetime tokens', note: 'since 14 Jul' },
	{ value: '14.2K', label: 'Peak day', note: '2 Aug' },
	{ value: '4m 12s', label: 'Longest task', note: 'salah-daily-tracker' },
	{ value: '9 days', label: 'Longest streak', note: '1–9 Aug' },
	{ value: '3 days', label: 'Current streak', note: 'best is 9 — 6 to go', current: true },
];

describe('ProfileHeader and ProfileAvatar', () => {
	it('puts the 56px picture beside the name as the page’s title', async () => {
		await render(ProfileHeader, { name: 'Mario Prasetyo', line: '@mario · Jakarta · joined 14 Jul 2026 · Free plan' });
		await expect.element(page.getByRole('heading', { level: 1, name: 'Mario Prasetyo' })).toBeVisible();
		const avatar = page.getByText('MP').element().parentElement!;
		expect([box(avatar).width, getComputedStyle(avatar).borderRadius]).toEqual([56, '10px']);
		expect(getComputedStyle(page.getByText(/^@mario/).element()).fontFamily).toMatch(/Geist Mono/);
	});

	it('is 40 with corners of 6 by the photo controls', async () => {
		await render(ProfileAvatar, { name: 'Mario Prasetyo', size: 40 });
		const avatar = page.getByText('MP').element().parentElement!;
		expect([box(avatar).width, getComputedStyle(avatar).borderRadius]).toEqual([40, '6px']);
	});
});

describe('StatStrip', () => {
	it('shows each number over what it is, 107 tall, the current streak on the surface', async () => {
		await page.viewport(1512, 862);
		await render(StatStrip, { tiles });
		const lifetime = page.getByText('186.4K').element();
		const style = getComputedStyle(lifetime);
		expect([style.fontSize, style.letterSpacing]).toEqual(['28px', '-0.6px']);
		const tile = lifetime.closest('div')!;
		expect(box(tile).height).toBe(107);
		expect(getComputedStyle(page.getByText('3 days').element().closest('div')!).backgroundColor).toBe(tokenBackground('nm-surface'));
		await expect.element(page.getByRole('term').first()).toHaveTextContent('Lifetime tokens');
	});
});

describe('ActivityHeatmap', () => {
	it('draws a year of 11px days 3 apart, the busiest in the strongest shade', async () => {
		const map = heatmap([{ date: '2026-10-21', tokens: 900 }, { date: '2026-10-20', tokens: 100 }], '2026-10-21', 'en');
		await render(ActivityHeatmap, { label: '1K tokens in the last year', weeks: map.weeks, months: map.months, days: ['Mon', 'Wed', 'Fri'], note: 'Most active Wed', less: 'Less', more: 'More' });
		const busiest = document.querySelector('[title="2026-10-21 · 900"]')!;
		expect([box(busiest).width, box(busiest).height]).toEqual([11, 11]);
		expect(getComputedStyle(busiest).backgroundColor).toBe(tokenBackground('nm-heat-4'));
		const before = document.querySelector('[title="2026-10-20 · 100"]')!;
		expect(box(busiest).top - box(before).top).toBe(14);
		await expect.element(page.getByRole('img', { name: '1K tokens in the last year' })).toBeVisible();
		await expect.element(page.getByText('Most active Wed')).toBeVisible();
	});
});

describe('the breakdowns', () => {
	it('draws each model’s share as a 4px bar against the most used', async () => {
		await render(UsageBars, { bars: [{ label: 'deepseek-flash', value: '13.5K · 62%', share: 1 }, { label: 'gemini-flash', value: '5.2K · 24%', share: 0.5 }], empty: 'None yet' });
		const fills = [...document.querySelectorAll('li span.block')];
		expect(box(fills[0]).height).toBe(4);
		expect(Math.round(box(fills[1]).width * 2)).toBe(Math.round(box(fills[0]).width));
		expect(getComputedStyle(fills[0]).backgroundColor).toBe(tokenBackground('nm-text-2'));
	});

	it('draws each crew member’s bar in their own colour', async () => {
		await render(CrewUsage, { crew: [{ agent: 'dana', name: 'Dana', count: '31', share: 1 }], empty: 'None yet' });
		const bar = page.getByRole('listitem').element().querySelector('span[style]')!;
		expect(box(bar).height).toBe(8);
		expect(getComputedStyle(bar).backgroundColor).toBe(tokenBackground('crew-dana-bg'));
		expect(getComputedStyle(page.getByText('31').element()).color).toBe(tokenColor('nm-text-2'));
	});

	it('says so when there’s nothing to show', async () => {
		await render(CrewUsage, { crew: [], empty: 'No chats with the crew yet.' });
		await expect.element(page.getByText('No chats with the crew yet.')).toBeVisible();
	});
});

describe('the showcase', () => {
	it('is a featured project, 76 tall, with its tools at the end', async () => {
		await render(ShowcaseCard, { name: 'salah-daily-tracker', status: 'Live', meta: 'built with Koda · 4m 12s' });
		const card = page.getByRole('article', { name: 'salah-daily-tracker' }).element();
		expect(box(card).height).toBe(76);
		await expect.element(page.getByText('Live')).toBeVisible();
	});

	it('has room for one more as an outlined 44px button', async () => {
		const onclick = vi.fn();
		await render(ShowcaseAdd, { label: 'Add a live project', onclick });
		const button = page.getByRole('button', { name: 'Add a live project' });
		expect(box(button.element()).height).toBe(44);
		await button.click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});

describe('SaveBar', () => {
	it('says what isn’t saved in the warning colour, with Discard and Save', async () => {
		const ondiscard = vi.fn();
		await render(SaveBar, { pending: '1 unsaved change', dirty: true, discard: 'Discard', save: 'Save', ondiscard });
		expect(getComputedStyle(page.getByText('1 unsaved change').element()).color).toBe(tokenColor('nm-warning'));
		await page.getByRole('button', { name: 'Discard' }).click();
		expect(ondiscard).toHaveBeenCalledOnce();
	});

	it('waits while nothing has changed', async () => {
		await render(SaveBar, { pending: '1 unsaved change', dirty: false, discard: 'Discard', save: 'Save' });
		await expect.element(page.getByRole('button', { name: 'Save' })).toBeDisabled();
		await expect.element(page.getByRole('button', { name: 'Discard' })).toBeDisabled();
	});
});
