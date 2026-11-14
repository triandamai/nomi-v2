import { describe, expect, it } from 'vitest';
import { budgetShare, settingsFromForm } from './memorySettings';

const form = (entries: Record<string, string>) => {
	const data = new FormData();
	for (const [k, v] of Object.entries(entries)) data.set(k, v);
	return data;
};

describe('budgetShare', () => {
	it('is a whole percent, capped at 100', () => {
		expect(budgetShare(0, 8000)).toBe(0);
		expect(budgetShare(2000, 8000)).toBe(25);
		expect(budgetShare(9000, 8000)).toBe(100);
		expect(budgetShare(10, 0)).toBe(0);
	});
});

describe('settingsFromForm', () => {
	it('reads ticked switches as on and the budgets as numbers', () => {
		expect(settingsFromForm(form({ learn_from_chats: 'on', keep_profile: 'false', search_past_chats: 'true', memory_budget: '8000', profile_budget: '1000' }))).toEqual({
			update: { learn_from_chats: true, keep_profile: false, search_past_chats: true, memory_budget: 8000, profile_budget: 1000 },
		});
	});

	it('says which budget is out of range', () => {
		expect(settingsFromForm(form({ memory_budget: '10', profile_budget: '1000' }))).toEqual({ invalid: 'memory_budget' });
		expect(settingsFromForm(form({ memory_budget: '8000', profile_budget: 'lots' }))).toEqual({ invalid: 'profile_budget' });
	});
});
