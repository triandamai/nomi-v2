import { describe, expect, it } from 'vitest';
import { monthLinks, quiet, usageChart } from './usage';
import type { AdminUsage } from './api';

const usage = (patch: Partial<AdminUsage> = {}): AdminUsage => ({
	month: '2026-11',
	current_month: '2026-11',
	chat: { tokens: 0, input_tokens: 0, output_tokens: 0, cached_input_tokens: 0, own_key_tokens: 0, calls: 0, people: 0, spend_usd: 0 },
	embedding: { tokens: 0, calls: 0, estimated_calls: 0 },
	by_day: [],
	models: [],
	embedding_models: [],
	sandbox_minutes: [],
	...patch,
});

describe('usage by month', () => {
	it('draws every day of the month from zero, ticking the 1st and every 7th, against a round top', () => {
		const chart = usageChart(usage({ by_day: [{ date: '2026-11-03', chat_tokens: 1234, embedding_tokens: 50 }] }), 'en');
		expect(chart.bars).toHaveLength(30);
		expect(chart.max).toBe(2000);
		expect(chart.axis).toEqual(['2K', '1K', '0']);
		expect(chart.bars.filter((bar) => bar.tick).map((bar) => bar.tick)).toEqual(['1', '7', '14', '21', '28']);
		expect(chart.bars[2]).toMatchObject({ key: '2026-11-03', value: 1234, tooltip: ['Tue 3 Nov', '1,234 chat tokens', '50 embedding tokens'] });
		expect(chart.days).toEqual([{ date: '2026-11-03', label: 'Tue 3 Nov', chat: '1,234', embedding: '50' }]);
	});

	it('steps back without end, and forward only up to this month', () => {
		expect(monthLinks(usage())).toEqual({ previous: '?month=2026-10', next: null, label: 'November 2026' });
		expect(monthLinks(usage({ month: '2026-09' }))).toEqual({ previous: '?month=2026-08', next: '?month=2026-10', label: 'September 2026' });
	});

	it('knows a month nobody used anything in', () => {
		expect(quiet(usage())).toBe(true);
		expect(quiet(usage({ embedding: { tokens: 10, calls: 1, estimated_calls: 0 } }))).toBe(false);
	});
});
