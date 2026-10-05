import { describe, expect, it } from 'vitest';
import { planChecklist, planExcerpt } from './planChecklist';

const PLAN = `# Bali, 6 days

- [x] Pick dates · Tue 10 – Sun 15 Mar
- [X] Split nights: **Ubud** then Canggu
- [ ] Set booking reminders
- [ ] Day-by-day [itinerary](https://example.com)
`;

describe('planChecklist', () => {
	it('reads task items and marks the first unfinished one as current', () => {
		expect(planChecklist(PLAN)).toEqual([
			{ text: 'Pick dates · Tue 10 – Sun 15 Mar', status: 'done' },
			{ text: 'Split nights: Ubud then Canggu', status: 'done' },
			{ text: 'Set booking reminders', status: 'in_progress' },
			{ text: 'Day-by-day itinerary', status: 'pending' },
		]);
	});
	it('finds nothing in a plan without task items', () => {
		expect(planChecklist('# Plan\n\nJust prose.\n- a bullet')).toEqual([]);
	});
});

describe('planExcerpt', () => {
	it('skips headings and joins the prose', () => {
		expect(planExcerpt('# Title\n\nFly midweek.\n\n- Stay in **Ubud**')).toBe('Fly midweek. Stay in Ubud');
	});
	it('clips long prose', () => {
		expect(planExcerpt('word '.repeat(100), 20)).toHaveLength(20);
	});
});
