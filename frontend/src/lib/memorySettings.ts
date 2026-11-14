// Settings → Memory: the switches, the budgets and the profile (GET|PUT /api/memory/settings and
// /api/memory/profile). The ranges mirror backend/crates/nomi-agent-core/src/memory_settings.rs.

export const MEMORY_BUDGET_RANGE = { min: 500, max: 100_000 } as const;
export const PROFILE_BUDGET_RANGE = { min: 100, max: 4_000 } as const;
const PERCENT = 100;

/** How much of a budget is in use, as a whole percent from 0 to 100. */
export function budgetShare(used: number, budget: number): number {
	if (!(budget > 0) || !(used > 0)) return 0;
	return Math.min(PERCENT, Math.round((used / budget) * PERCENT));
}

export interface SettingsUpdate {
	learn_from_chats: boolean;
	keep_profile: boolean;
	search_past_chats: boolean;
	memory_budget: number;
	profile_budget: number;
}

/** The settings form as the PUT body (switches left off or missing are off), or which budget is out of range. */
export function settingsFromForm(data: FormData): { update: SettingsUpdate } | { invalid: 'memory_budget' | 'profile_budget' } {
	const memory_budget = Number(data.get('memory_budget'));
	const profile_budget = Number(data.get('profile_budget'));
	const within = (v: number, range: { min: number; max: number }) => Number.isInteger(v) && v >= range.min && v <= range.max;
	if (!within(memory_budget, MEMORY_BUDGET_RANGE)) return { invalid: 'memory_budget' };
	if (!within(profile_budget, PROFILE_BUDGET_RANGE)) return { invalid: 'profile_budget' };
	// A Switch submits "true"/"false"; a checkbox "on" when ticked.
	const on = (name: string) => ['on', 'true'].includes(String(data.get(name)));
	return {
		update: {
			learn_from_chats: on('learn_from_chats'),
			keep_profile: on('keep_profile'),
			search_past_chats: on('search_past_chats'),
			memory_budget,
			profile_budget,
		},
	};
}
