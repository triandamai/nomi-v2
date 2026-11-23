import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import PhoneStatGrid from './PhoneStatGrid.svelte';
import PhonePlanSummary from './PhonePlanSummary.svelte';
import PhoneDayChart from './PhoneDayChart.svelte';
import PhonePlanOption from './PhonePlanOption.svelte';
import type { DayBar } from './DayBars.svelte';

// Flow / Mobile — You & Settings, steps 8 and 9 (Plan & usage, Compare plans).

afterEach(() => {
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const button = (label: string) => createRawSnippet(() => ({ render: () => `<button type="button">${label}</button>` }));

describe('PhoneStatGrid', () => {
	it('puts two numbers to a row', async () => {
		await page.viewport(390, 844);
		await render(PhoneStatGrid, {
			tiles: [
				{ label: 'Spend at list price', value: '$0.42', note: 'not what you pay' },
				{ label: 'Requests', value: '86', note: 'across 3 models' },
				{ label: 'Tokens in', value: '241K', note: 'prompts + memory' },
			],
		});
		const [spend, requests, tokens] = ['$0.42', '86', '241K'].map((value) => box(page.getByText(value, { exact: true }).element()));
		expect(Math.round(spend.top)).toBe(Math.round(requests.top));
		expect(tokens.top).toBeGreaterThan(spend.bottom);
		await expect.element(page.getByRole('term').filter({ hasText: 'Requests' })).toBeVisible();
	});
});

describe('PhonePlanSummary', () => {
	it('leads with the plan, meters what’s used, and holds its action', async () => {
		await render(PhonePlanSummary, { plan: 'Free', line: '1M tokens a month', used: 0.31, usedLabel: '312K / 1M tokens · 31% used', resets: 'resets 1 Dec · 8 days', action: button('Compare plans') });
		await expect.element(page.getByRole('region', { name: 'Free' })).toBeVisible();
		await expect.element(page.getByRole('meter', { name: '312K / 1M tokens · 31% used' })).toHaveAttribute('aria-valuenow', '31');
		await expect.element(page.getByText('resets 1 Dec · 8 days')).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Compare plans' })).toBeVisible();
	});
});

describe('PhoneDayChart', () => {
	it('draws the days up to today, today in the accent, and names each', async () => {
		const bars: DayBar[] = [
			{ day: 1, value: 10, share: 0.5, state: 'past' },
			{ day: 2, value: 0, share: 0, state: 'empty' },
			{ day: 3, value: 20, share: 1, state: 'today' },
			{ day: 4, value: 0, share: 0, state: 'future' },
		];
		await render(PhoneDayChart, { label: 'Each day of November', bars, describe: (bar) => `${bar.day} Nov: ${bar.value}` });
		const list = page.getByRole('list', { name: 'Each day of November' });
		await expect.element(list).toBeVisible();
		expect(list.element().children).toHaveLength(3);
		await expect.element(page.getByText('3 Nov: 20')).toBeInTheDocument();
		expect(list.element().lastElementChild!.querySelector('.bg-accent')).not.toBeNull();
	});
});

describe('PhonePlanOption', () => {
	it('marks the person’s own plan, and gives another its action', async () => {
		await render(PhonePlanOption, { name: 'Free', price: 'Rp 0', line: '1M tokens a month', yours: 'Your plan' });
		await expect.element(page.getByRole('article', { name: 'Free' })).toBeVisible();
		await expect.element(page.getByText('Your plan')).toBeVisible();
		document.body.innerHTML = '';
		await render(PhonePlanOption, { name: 'Pro', price: 'Rp 99.000', line: '10M tokens a month', children: button('Upgrade to Pro') });
		await expect.element(page.getByRole('heading', { name: 'Pro' })).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Upgrade to Pro' })).toBeVisible();
	});
});
