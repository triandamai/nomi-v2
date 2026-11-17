import { describe, expect, it } from 'vitest';
import type { ProjectUsage } from '$lib/adapters/api';
import { rupiah, usageView } from './usage';

const usage = (over: Partial<ProjectUsage> = {}): ProjectUsage => ({
	month: '2026-11',
	current_month: '2026-11',
	timezone: 'Asia/Jakarta',
	counted_since: '2026-11-17T03:00:00Z',
	tokens: 1_350_000,
	input_tokens: 1_250_000,
	output_tokens: 100_000,
	cached_input_tokens: 0,
	own_key_tokens: 50_000,
	calls: 3,
	spend_usd: 5.1,
	by_model: [
		{ admin_model_id: 'm1', label: 'Nomi Standard', provider: 'anthropic', model_id: 'claude-x', source: 'nomi', tokens: 1_300_000, calls: 2, spend_usd: 5.1 },
		{ admin_model_id: null, label: 'My Claude key', provider: 'anthropic', model_id: 'claude-x', source: 'own_key', tokens: 50_000, calls: 1, spend_usd: 0 },
	],
	by_person: [{ user_id: 'u1', name: 'Budi Santoso', tokens: 1_100_000, calls: 1, spend_usd: 4.5 }],
	shows_people: true,
	usd_to_idr: 16_000,
	...over,
});

describe('a project’s tokens and cost', () => {
	it('writes rupiah with Indonesian grouping', () => {
		expect(rupiah(81_600)).toBe('Rp 81.600');
		expect(rupiah(0)).toBe('Rp 0');
	});

	it('says the month’s spend in rupiah with the dollars beside it, its tokens and calls', () => {
		const view = usageView(usage(), 'en');
		expect(view.month).toBe('November 2026');
		expect(view.current).toBe(true);
		expect(view.spend).toBe('Rp 81.600');
		expect(view.spendUsd).toBe('$5.10');
		expect(view.tokens).toBe('1.3M');
		expect(view.calls).toBe('3');
		expect(view.ownKey).toBe('50K');
		expect(view.empty).toBe(false);
	});

	it('marks own-key models and lists people only for admins', () => {
		const view = usageView(usage(), 'en');
		expect(view.models.map((row) => [row.label, row.ownKey, row.tokens, row.spend])).toEqual([
			['Nomi Standard', false, '1.3M', '$5.10'],
			['My Claude key', true, '50K', '$0'],
		]);
		expect(view.people).toEqual([{ id: 'u1', name: 'Budi Santoso', tokens: '1.1M', spend: '$4.50' }]);
		expect(usageView(usage({ shows_people: false, by_person: [] }), 'en').people).toBeNull();
	});

	it('says since when it counts, or that it hasn’t started', () => {
		expect(usageView(usage(), 'en').since).toBe('17 Nov 2026');
		expect(usageView(usage(), 'id').since).toBe('17 Nov 2026');
		expect(usageView(usage({ counted_since: null }), 'en').since).toBeNull();
	});

	it('reads an empty month as empty, with no own-key line', () => {
		const view = usageView(usage({ calls: 0, tokens: 0, spend_usd: 0, own_key_tokens: 0, by_model: [], by_person: [], month: '2026-10' }), 'en');
		expect(view.empty).toBe(true);
		expect(view.ownKey).toBeNull();
		expect(view.current).toBe(false);
	});
});
