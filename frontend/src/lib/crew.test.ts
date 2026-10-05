import { describe, expect, it } from 'vitest';
import { buildCrew, type CrewActivity } from './crew';

function activity(target: string, status: string, task: string): CrewActivity {
	return { id: `${target}-${task}`, target_agent_type: target, status, task, result: null, error: null };
}

describe('buildCrew', () => {
	it('always lists the five core members, Nomi first', () => {
		const crew = buildCrew({ activity: [], messageAuthors: [], nomiWorking: false, nomiStatus: null });
		expect(crew.map((m) => m.key)).toEqual(['nomi', 'money', 'coding', 'planning', 'personality']);
		expect(crew[0].status).toBe('Ready when you are');
		expect(crew[1]).toMatchObject({ working: false, involved: false, status: 'Standing by' });
	});

	it('marks a member working from an in-flight delegation and shows its task', () => {
		const crew = buildCrew({
			activity: [activity('money', 'processing', 'Summarize March spending')],
			messageAuthors: [],
			nomiWorking: false,
			nomiStatus: null,
		});
		expect(crew.find((m) => m.key === 'money')).toMatchObject({ working: true, involved: true, status: 'Summarize March spending' });
	});

	it('uses the newest delegation for the status line (activity arrives newest first)', () => {
		const crew = buildCrew({
			activity: [activity('planning', 'completed', 'Set reminders'), activity('planning', 'failed', 'Draft itinerary')],
			messageAuthors: [],
			nomiWorking: false,
			nomiStatus: null,
		});
		expect(crew.find((m) => m.key === 'planning')?.status).toBe('Done: Set reminders');
	});

	it('maps display names onto core members and adds unknown agents to the end', () => {
		const crew = buildCrew({ activity: [], messageAuthors: ['Money', null, 'travel_scout'], nomiWorking: false, nomiStatus: null });
		expect(crew.find((m) => m.key === 'money')?.involved).toBe(true);
		expect(crew.at(-1)).toMatchObject({ key: 'travel_scout', name: 'Travel scout', involved: true });
	});

	it("shows Nomi's live phase while a turn is running", () => {
		const crew = buildCrew({ activity: [], messageAuthors: [], nomiWorking: true, nomiStatus: 'thinking…' });
		expect(crew[0]).toMatchObject({ working: true, status: 'thinking…' });
	});
});
