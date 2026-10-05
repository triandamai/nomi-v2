import { describe, expect, it } from 'vitest';
import { crewByActivity, crewPreview, type CrewRosterMember } from './crew';

function member(agent_type: string, state: CrewRosterMember['state']): CrewRosterMember {
	return { agent_type, name: agent_type, role: '', is_dynamic: false, shape: null, tone: null, motion: null, state, status: '' };
}

describe('crew preview', () => {
	const crew = [
		member('chitchat', 'idle'),
		member('money', 'idle'),
		member('reminders', 'done'),
		member('files', 'idle'),
		member('planning', 'working'),
		member('workspace', 'waiting'),
	];

	it('puts the busiest first and keeps the roster order otherwise', () => {
		expect(crewByActivity(crew).map((m) => m.agent_type)).toEqual(['planning', 'workspace', 'reminders', 'chitchat', 'money', 'files']);
	});

	it('shows four and counts the rest', () => {
		const { shown, more } = crewPreview(crew);
		expect(shown.map((m) => m.agent_type)).toEqual(['planning', 'workspace', 'reminders', 'chitchat']);
		expect(more).toBe(2);
	});

	it('shows everyone when the crew is small', () => {
		expect(crewPreview(crew.slice(0, 3))).toEqual({ shown: crewByActivity(crew.slice(0, 3)), more: 0 });
	});
});
