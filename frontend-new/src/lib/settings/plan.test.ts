import { describe, expect, it } from 'vitest';
import type { UsageMonth } from '$lib/adapters/api';
import { allowance, dayBars, daysIn, fitNote, modelSpend, monthName, shiftMonth, usageCsv, usageStats, usd } from './plan';

const usage = (extra: Partial<UsageMonth> = {}): UsageMonth => ({
	plan: { id: 'free', name: 'Free', card_tone: 'plain', monthly_tokens: 1_000_000, custom_quota: false },
	month: '2026-10',
	current_month: '2026-10',
	timezone: 'Asia/Jakarta',
	tokens_used: 21_000,
	input_tokens: 20_600,
	output_tokens: 1_200,
	cached_input_tokens: 0,
	own_key_tokens: 1_100,
	calls: 64,
	spend_usd: 0.11,
	by_day: [
		{ date: '2026-10-02', tokens: 1_000, spend_usd: 0.01 },
		{ date: '2026-10-19', tokens: 4_100, spend_usd: 0.03 },
		{ date: '2026-10-21', tokens: 900, spend_usd: 0.004 },
	],
	by_model: [
		{ admin_model_id: 'a', label: 'deepseek-flash', provider: 'deepseek', model_id: 'deepseek-flash', source: 'nomi', tokens: 13_500, calls: 40, spend_usd: 0.06 },
		{ admin_model_id: null, label: 'llama', provider: 'ollama', model_id: 'llama', source: 'own_key', tokens: 1_100, calls: 3, spend_usd: 0 },
	],
	...extra,
});

describe('months', () => {
	it('count their days, name themselves and step back and forth', () => {
		expect([daysIn('2026-10'), daysIn('2026-02'), daysIn('2028-02')]).toEqual([31, 28, 29]);
		expect(monthName('2026-10', 'en')).toBe('October 2026');
		expect(monthName('2026-10', 'id')).toBe('Oktober 2026');
		expect([shiftMonth('2026-01', -1), shiftMonth('2026-12', 1)]).toEqual(['2025-12', '2027-01']);
	});
});

describe('the allowance', () => {
	it('is what’s used, what this pace comes to by the month’s end, and when it starts again', () => {
		expect(allowance(usage(), '2026-10-21')).toEqual({ used: 0.021, projected: 31_000, projectedShare: 0.031, resets: '2026-11-01', daysLeft: 11, current: true });
	});

	it('is just what was used for a month that’s over', () => {
		expect(allowance(usage({ month: '2026-09' }), '2026-10-21')).toMatchObject({ projected: 21_000, current: false });
	});

	it('never shows more than full', () => {
		expect(allowance(usage({ tokens_used: 2_000_000 }), '2026-10-21')).toMatchObject({ used: 1, projectedShare: 1 });
	});
});

describe('day by day', () => {
	it('is a bar for each day of the month: today in the accent, the days after it a stub', () => {
		const chart = dayBars(usage(), '2026-10-21', 'tokens');
		expect(chart.bars).toHaveLength(31);
		expect(chart.bars[18]).toEqual({ day: 19, value: 4_100, share: 1, state: 'past' });
		expect(chart.bars[20]).toMatchObject({ day: 21, state: 'today' });
		expect(chart.bars[0]).toMatchObject({ day: 1, value: 0, state: 'empty' });
		expect(chart.bars[21]).toMatchObject({ day: 22, state: 'future' });
		expect(chart.peak).toEqual({ value: 4_100, date: '2026-10-19' });
	});

	it('shows spend instead of tokens when asked', () => {
		expect(dayBars(usage(), '2026-10-21', 'spend').peak).toEqual({ value: 0.03, date: '2026-10-19' });
	});

	it('has no today or days ahead in a month that’s over', () => {
		const chart = dayBars(usage({ month: '2026-09' }), '2026-10-21', 'tokens');
		expect(chart.bars).toHaveLength(30);
		expect(chart.bars.some((bar) => bar.state === 'today' || bar.state === 'future')).toBe(false);
	});
});

describe('the month’s figures', () => {
	it('write dollars at list price', () => {
		expect([usd(0.11), usd(12.5), usd(0.004), usd(0)]).toEqual(['$0.11', '$12.50', '<$0.01', '$0']);
	});

	it('are spend, tokens in and out, and requests across the models', () => {
		expect(usageStats(usage())).toEqual([
			{ label: 'Spend at list price', value: '$0.11', note: 'not what you pay' },
			{ label: 'Tokens in', value: '20.6K', note: 'prompts + memory' },
			{ label: 'Tokens out', value: '1.2K', note: 'replies' },
			{ label: 'Requests', value: '64', note: 'across 2 models' },
		]);
	});

	it('split by model, with whose key paid', () => {
		expect(modelSpend(usage())).toEqual([
			{ label: 'deepseek-flash', value: '$0.06', share: 1, note: '13.5K tok · 92% · Nomi’s allowance' },
			{ label: 'llama', value: '$0', share: 1_100 / 13_500, note: '1.1K tok · 8% · your own key' },
		]);
	});
});

describe('the fit check', () => {
	const plans = [
		{ name: 'Free', monthly_tokens: 1_000_000 },
		{ name: 'Pro', monthly_tokens: 10_000_000 },
	];

	it('says whether this pace fits the plan they’re on', () => {
		expect(fitNote(31_000, { name: 'Free', monthly_tokens: 1_000_000 }, plans)).toBe('At your pace you’ll use about 31K this month. Free covers it.');
	});

	it('names the plan that would, when theirs won’t', () => {
		expect(fitNote(3_000_000, { name: 'Free', monthly_tokens: 1_000_000 }, plans)).toBe('At your pace you’ll use about 3M this month, more than Free’s 1M. Pro would cover it.');
		expect(fitNote(30_000_000, { name: 'Free', monthly_tokens: 1_000_000 }, plans)).toBe('At your pace you’ll use about 30M this month, more than Free’s 1M.');
	});
});

describe('the usage export', () => {
	it('is a row a day, then a row a model, with exact figures', () => {
		expect(usageCsv(usage()).split('\n')).toEqual([
			'date,tokens,spend_usd',
			'2026-10-02,1000,0.01',
			'2026-10-19,4100,0.03',
			'2026-10-21,900,0.004',
			'',
			'model,source,tokens,requests,spend_usd',
			'deepseek-flash,nomi,13500,40,0.06',
			'llama,own_key,1100,3,0',
		]);
	});

	it('quotes a model name with a comma or a quote in it', () => {
		const odd = usage({ by_day: [], by_model: [{ ...usage().by_model[0], label: 'gpt "4o", mini' }] });
		expect(usageCsv(odd).split('\n').at(-1)).toBe('"gpt ""4o"", mini",nomi,13500,40,0.06');
	});
});
