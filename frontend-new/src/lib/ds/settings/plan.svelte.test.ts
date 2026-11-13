import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { KeyRound } from '@lucide/svelte';
import PlanSummary from './PlanSummary.svelte';
import MetaList from './MetaList.svelte';
import MonthSwitch from './MonthSwitch.svelte';
import DayBars from './DayBars.svelte';
import PlanOption from './PlanOption.svelte';
import FitNote from './FitNote.svelte';
import SettingRow from './SettingRow.svelte';
import SettingsTip from './SettingsTip.svelte';
import StatStrip from '../profile/StatStrip.svelte';
import UsageBars from '../profile/UsageBars.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Plan & usage's pieces (App / Settings / Plan & usage C0og49), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('PlanSummary', () => {
	it('shows what’s used in the accent and the pace in heat-2 on a 6px track', async () => {
		await render(PlanSummary, {
			label: 'Your plan',
			tag: 'Free plan',
			plan: 'Free',
			line: 'Rp 0 / month · 1M tokens a month',
			allowance: { label: 'Allowance', value: '21.8K / 1M tokens' },
			used: 0.25,
			projected: 0.5,
			legend: { used: '25% used', projected: '~500K projected by 31 Oct', resets: 'resets Sun 1 Nov · 11 days' },
		});
		const meter = page.getByRole('meter', { name: 'Allowance' }).element();
		expect(box(meter).height).toBe(6);
		const [projected, used] = [...meter.querySelectorAll('span')];
		expect(getComputedStyle(used).backgroundColor).toBe(tokenBackground('nm-accent'));
		expect(getComputedStyle(projected).backgroundColor).toBe(tokenBackground('nm-heat-2'));
		expect(Math.round(box(projected).width)).toBe(Math.round(box(used).width * 2));
		expect(getComputedStyle(page.getByText('Free', { exact: true }).element()).fontSize).toBe('40px');
	});
});

describe('the billing facts and the month', () => {
	it('lists facts in pairs, the value in mono, 32 tall', async () => {
		await render(MetaList, { label: 'Billing', rows: [{ label: 'Allowance resets', value: 'Sun 1 Nov' }, { label: 'Member since', value: '14 Jul 2026' }] });
		expect(box(page.getByRole('term').first().element().parentElement!).height).toBe(32);
		expect(getComputedStyle(page.getByText('Sun 1 Nov').element()).fontFamily).toMatch(/Geist Mono/);
	});

	it('steps back a month, and waits on the current one', async () => {
		await render(MonthSwitch, { month: 'October 2026', previous: { href: '?month=2026-09', label: 'September 2026' }, nextLabel: 'No later month' });
		await expect.element(page.getByRole('link', { name: 'September 2026' })).toHaveAttribute('href', '?month=2026-09');
		await expect.element(page.getByRole('link', { name: 'No later month' })).toHaveAttribute('aria-disabled', 'true');
		expect(box(page.getByText('October 2026').element().parentElement!).height).toBe(32);
	});
});

describe('DayBars', () => {
	it('draws the busiest day full height, today in the accent, the days ahead as stubs', async () => {
		const bars = [
			{ day: 1, value: 4, share: 1, state: 'past' as const },
			{ day: 2, value: 2, share: 0.5, state: 'today' as const },
			{ day: 3, value: 0, share: 0, state: 'future' as const },
		];
		await render(DayBars, { label: 'Tokens each day', bars, scale: ['4', '2', '0'], today: 'today', describe: (bar: { day: number; value: number }) => `${bar.day}: ${bar.value}` });
		const [first, second, third] = [...document.querySelectorAll('li')].map((item) => item.querySelector('span:not(.sr-only)')!);
		expect([box(first).height, box(second).height, box(third).height]).toEqual([140, 70, 2]);
		expect(getComputedStyle(first).backgroundColor).toBe(tokenBackground('nm-text-2'));
		expect(getComputedStyle(second).backgroundColor).toBe(tokenBackground('nm-accent'));
		expect(getComputedStyle(page.getByText('today').element()).color).toBe(tokenColor('nm-text'));
	});
});

describe('PlanOption and FitNote', () => {
	it('is a plan to compare: its price in mono, what it includes with checks', async () => {
		await render(PlanOption, { name: 'Free', tag: { label: 'Current', kind: 'go' }, price: 'Rp 0', per: '/ month', description: 'Everything Nomi does.', features: ['1M tokens a month', 'The whole crew'] });
		await expect.element(page.getByRole('article', { name: 'Free' })).toBeVisible();
		expect(getComputedStyle(page.getByText('Rp 0').element()).fontFamily).toMatch(/Geist Mono/);
		expect(page.getByRole('listitem').all()).toHaveLength(2);
		expect(getComputedStyle(page.getByText('Current').element()).color).toBe(tokenColor('nm-accent'));
	});

	it('says whether the plan fits on the accent tint, 40 tall', async () => {
		await page.viewport(1512, 862);
		await render(FitNote, { text: 'At your pace you’ll use about 31K this month. Free covers it.' });
		const note = page.getByRole('status').element();
		expect([box(note).height, getComputedStyle(note).backgroundColor]).toEqual([40, tokenBackground('nm-accent-bg')]);
	});
});

describe('the variants Plan & usage adds', () => {
	it('leads a row with its icon, 12 before the text', async () => {
		await render(SettingRow, { title: 'DeepSeek', description: 'sk-…a91f', mono: true, icon: KeyRound });
		const icon = document.querySelector('svg')!;
		expect(box(page.getByText('DeepSeek').element()).left - box(icon).right).toBe(12);
	});

	it('explains with the quiet info glyph', async () => {
		await render(SettingsTip, { text: 'Spend is shown at list price.', kind: 'info' });
		expect(getComputedStyle(document.querySelector('svg')!.parentElement!).color).toBe(tokenColor('nm-text-3'));
	});

	it('shows small figures: the name first at 12, the figure at 15 in mono, 85 tall', async () => {
		await page.viewport(1512, 862);
		await render(StatStrip, { small: true, tiles: [{ value: '$0.11', label: 'Spend at list price', note: 'the crew’s work at list price' }] });
		const figure = page.getByText('$0.11').element();
		expect(getComputedStyle(figure).fontSize).toBe('15px');
		expect(box(figure.closest('div')!).height).toBe(85);
	});

	it('puts a note under a model’s bar, the spend stronger', async () => {
		await render(UsageBars, { bars: [{ label: 'deepseek-flash', value: '$0.06', share: 1, note: '13.5K tok · 92% · Nomi’s allowance' }], empty: 'None' });
		expect(getComputedStyle(page.getByText('$0.06').element()).color).toBe(tokenColor('nm-text-2'));
		await expect.element(page.getByText('13.5K tok · 92% · Nomi’s allowance')).toBeVisible();
	});
});
