import { describe, expect, it } from 'vitest';
import type { CrewMember } from '$lib/adapters/api';
import { chosenMember, crewGroups, crewMeta, modelRole } from './crew';

const member = (agent_type: string, name: string, role = '', extra: Partial<CrewMember> = {}): CrewMember => ({ agent_type, name, role, is_dynamic: false, state: 'idle', status: '', ...extra });
const crew = [
	member('chitchat', 'Nomi', 'Talks with you, routes work'),
	member('money', 'Dana', 'Money records', { state: 'working', status: 'Summarising March' }),
	member('supervisor', 'Supervisor', 'Watches the crew'),
	member('coding', 'Koda', 'Builds projects'),
	member('dyn-1', 'Bima', 'Investor updates', { is_dynamic: true }),
];

describe('the crew in Settings', () => {
	it('puts Nomi and the Supervisor first, then the specialists, then custom members', () => {
		expect(crewGroups(crew, '').map((group) => [group.key, group.members.map((one) => one.name)])).toEqual([
			['core', ['Nomi', 'Supervisor']],
			['specialists', ['Dana', 'Koda']],
			['custom', ['Bima']],
		]);
	});

	it('finds members by name or what they do, and drops empty groups', () => {
		expect(crewGroups(crew, 'build').map((group) => [group.key, group.members.map((one) => one.name)])).toEqual([['specialists', ['Koda']]]);
		expect(crewGroups(crew, ' DANA ')).toHaveLength(1);
		expect(crewGroups(crew, 'nobody')).toEqual([]);
	});

	it('opens the member asked for, or Nomi', () => {
		expect(chosenMember(crew, 'coding')?.name).toBe('Koda');
		expect(chosenMember(crew, 'gone')?.name).toBe('Nomi');
		expect(chosenMember([], null)).toBeNull();
	});

	it('says how many there are and how many are at work', () => {
		expect(crewMeta(crew)).toBe('5 members · 1 working');
		expect(crewMeta([crew[0]])).toBe('1 member');
	});

	it('knows whose model can be chosen here', () => {
		expect([modelRole('chitchat'), modelRole('coding'), modelRole('money')]).toEqual(['chat', 'coding', null]);
	});
});
